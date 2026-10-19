<template>
  <div
    class="photo-map-view group/map relative flex-1 overflow-hidden"
    @mouseenter="uiStore.setMapActive(true)"
    @mouseleave="uiStore.setMapActive(false)"
  >
    <div v-if="loading" class="absolute inset-0 z-50 flex items-center justify-center bg-base-200/50">
      <span class="loading loading-spinner loading-md text-primary"></span>
    </div>
    <div ref="mapEl" class="h-full w-full"></div>
    <div class="absolute top-2 left-2 z-500 flex cursor-pointer rounded-box bg-base-100/30 opacity-0 pointer-events-none transition-opacity duration-150 group-hover/map:bg-base-100/70 group-hover/map:opacity-100 group-hover/map:pointer-events-auto">
      <TButton :icon="IconZoomOut" :tooltip="t('map.zoom_out')" :disabled="zoom <= 0" @click="zoomOut" />
      <TButton :icon="IconZoomIn" :tooltip="t('map.zoom_in')" :disabled="zoom >= activeMaxZoom" @click="zoomIn" />
      <TButton :icon="IconMapCenter" :tooltip="t('map.zoom_center')" @click="isQueryMap ? fitBounds() : zoomCenter()" />
      <TButton
        :icon="config.settings.mapTheme === 0 ? IconMapDefault : IconMapSatellite"
        :tooltip="t(config.settings.mapTheme === 0 ? 'map.standard' : 'map.satellite')"
        @click="toggleMap"
      />
      <TButton v-if="showAppleMapsButton" :icon="IconExternal" :tooltip="t('file_info.open_apple_maps')" @click="openAppleMaps" />
    </div>
    <div v-if="isQueryMap && !loading" class="absolute top-2 right-2 z-500 pointer-events-none rounded-box bg-base-100/60 px-2 py-1 text-xs text-base-content/70">
      {{ points.length > 0 ? t('map.photo_count', { count: totalCount.toLocaleString() }) : t('map.no_photos_in_view') }}
    </div>
  </div>
</template>

<script setup>
import { MAP_DETAIL_ZOOM, MAP_DETAIL_LIMIT, MAP_MARKER_FADE_MS, MAP_SINGLE_POINT_FIT_ZOOM } from '@/common/constants';
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import L from 'leaflet'
import 'leaflet/dist/leaflet.css'

import { config } from '@/common/config'
import { createTileLayerGroup, getGlobalMapTheme, getMapTheme } from '@/common/mapProviders'
import {
  getCollectionQueryFileIds,
  getFilesByIds,
  getGpsMapPoints,
  getQueryFiles,
  getSmartQueryFileIds,
  openExternalUrl,
} from '@/common/api'
import { getThumbUrl, isMac } from '@/common/utils'
import { useUIStore } from '@/stores/uiStore'
import { IconExternal, IconMapCenter, IconMapDefault, IconMapSatellite, IconZoomIn, IconZoomOut } from '@/common/icons'
import TButton from '@/components/TButton.vue'

const props = defineProps({
  queryParams: { type: Object, default: null },
  querySource: { type: String, default: 'query' },
  collectionId: { type: Number, default: null },
  fileIds: { type: Array, default: () => [] },
  restoreView: { type: Object, default: null },
  active: { type: Boolean, default: true },
  lat: { type: Number, default: 0 },
  lon: { type: Number, default: 0 },
  label: { type: String, default: 'Lap' },
})
const emit = defineEmits(['open-cluster', 'select-file', 'preview-file', 'restored'])

const { t } = useI18n()
const uiStore = useUIStore()
const mapEl = ref(null)
const loading = ref(true)
const points = ref([])
const zoom = ref(2)
const singleMarker = ref(null)
const totalCount = computed(() => points.value.reduce((sum, point) => sum + point.count, 0))
const isQueryMap = computed(() => props.queryParams !== null)
const showAppleMapsButton = computed(() => !isQueryMap.value && isMac && validLatLon(props.lat, props.lon))

const activeMaxZoom = ref(19)

let map = null
let markerLayer = null
let tileLayer = null
let resizeObserver = null
let tileErrorFallbackTriggered = false
let pointRequestToken = 0
let detailRequestToken = 0
let detailTimer = null
let detailBounds = null
let clusterOpenRequestId = 0
let visibleFiles = []
let sourceFiles = null
let needsRefresh = false
let photoMarkers = new Map()
const retiringMarkers = new Map()
const fadeFrames = new Set()
let clusterWorker = null
let clusterRequestId = 0
let clusterRenderFrame = null
const clusterSources = new Map()

onMounted(async () => {
  map = L.map(mapEl.value, { center: [20, 0], zoom: 2, keyboard: false, zoomControl: false, maxZoom: activeMaxZoom.value })
  map.attributionControl.setPrefix('')
  L.control.scale({ position: 'bottomleft', imperial: false }).addTo(map)
  markerLayer = L.layerGroup().addTo(map)
  clusterWorker = new Worker(new URL('../common/mapClusters.worker.js', import.meta.url), { type: 'module' })
  clusterWorker.onmessage = ({ data }) => {
    if (!map || !props.active || data.requestId !== clusterRequestId) return
    if (data.error) { console.error('Photo clustering failed:', data.error); return }
    applyPhotoClusters(data.clusters)
  }
  map.on('zoomend', onMapChanged)
  map.on('moveend', onMapChanged)
  resizeObserver = new ResizeObserver(() => map?.invalidateSize())
  resizeObserver.observe(mapEl.value)
  updateTheme()
  window.addEventListener('keydown', handleMapKeyDown, true)
  if (props.active && isQueryMap.value) {
    await loadPoints(true)
  }
  else if (props.active) updateFromCoords()
  else loading.value = false
  requestAnimationFrame(() => map?.invalidateSize())
})

onBeforeUnmount(() => {
  uiStore.setMapActive(false)
  window.removeEventListener('keydown', handleMapKeyDown, true)
  if (detailTimer) clearTimeout(detailTimer)
  resizeObserver?.disconnect()
  clusterOpenRequestId++
  clusterWorker?.terminate()
  clusterWorker = null
  if (clusterRenderFrame !== null) cancelAnimationFrame(clusterRenderFrame)
  clusterSources.clear()
  pointRequestToken++
  detailRequestToken++
  for (const timer of retiringMarkers.values()) clearTimeout(timer)
  for (const frame of fadeFrames) cancelAnimationFrame(frame)
  retiringMarkers.clear()
  photoMarkers.clear()
  map?.remove()
  map = null
})

watch(() => [config.settings.mapTheme, config.settings.mapProvider, config.settings.tiandituToken], updateTheme)
watch(() => config.settings.mapMarkerSize, () => renderMarkers())
watch(() => [props.queryParams, props.querySource, props.collectionId, props.fileIds], () => {
  if (!props.active) {
    needsRefresh = true
    return
  }
  needsRefresh = false
  if (isQueryMap.value) void loadPoints(true)
  else updateFromCoords()
}, { deep: true })
watch(() => [props.lat, props.lon], () => {
  if (props.active && !isQueryMap.value) updateFromCoords()
})
watch(() => props.active, (active) => {
  if (!active) {
    clusterOpenRequestId++
    clusterRequestId++
    pointRequestToken++
    detailRequestToken++
    if (detailTimer) clearTimeout(detailTimer)
    return
  }
  void nextTick(async () => {
    if (!map || !props.active) return
    map.invalidateSize()
    if (props.restoreView || needsRefresh) {
      needsRefresh = false
      await loadPoints(true)
      return
    }
    zoom.value = map.getZoom()
    renderMarkers()
    scheduleDetailFetch()
  })
})

async function loadPoints(fitToResults) {
  if (!map) return
  clusterOpenRequestId++
  clusterRequestId++
  detailRequestToken++
  if (detailTimer) clearTimeout(detailTimer)
  detailBounds = null
  visibleFiles = []
  const token = ++pointRequestToken
  const restoreView = fitToResults ? props.restoreView : null
  loading.value = true
  try {
    const result = await getMapPoints()
    if (token !== pointRequestToken || !props.active) return
    points.value = result || []
    visibleFiles = []
    if (fitToResults) {
      if (restoreView) {
        map.setView([restoreView.lat, restoreView.lon], restoreView.zoom, { animate: false })
        emit('restored')
      } else if (points.value.length > 0) {
        fitMapToPoints()
      } else {
        map.setView([20, 0], 2, { animate: false })
      }
    }
    requestAnimationFrame(() => map?.invalidateSize())
    zoom.value = map.getZoom()
    renderMarkers()
    scheduleDetailFetch()
  } finally {
    if (token === pointRequestToken) loading.value = false
  }
}

function updateFromCoords() {
  if (!map) return
  if (singleMarker.value) {
    markerLayer.removeLayer(singleMarker.value)
    singleMarker.value = null
  }
  if (validLatLon(props.lat, props.lon)) {
    singleMarker.value = L.marker([props.lat, props.lon]).addTo(markerLayer)
    map.setView([props.lat, props.lon], zoom.value)
  } else {
    map.setView([0, 0], 2)
  }
  loading.value = false
}

function onMapChanged() {
  zoom.value = map.getZoom()
  if (!isQueryMap.value) return
  renderMarkers()
  scheduleDetailFetch()
}

function scheduleDetailFetch() {
  detailRequestToken++
  if (detailTimer) clearTimeout(detailTimer)
  if (zoom.value < MAP_DETAIL_ZOOM || !props.active) return
  detailTimer = setTimeout(fetchVisibleFiles, 200)
}

function bufferedMapBounds(padding) {
  const bounds = map.getPixelBounds()
  return L.latLngBounds(
    map.unproject(bounds.min.subtract([padding, padding])),
    map.unproject(bounds.max.add([padding, padding])),
  )
}

async function fetchVisibleFiles() {
  if (!map || !props.active || zoom.value < MAP_DETAIL_ZOOM) return
  // Reuse complete data until even the rendered buffer leaves the loaded area.
  if (detailBounds?.contains(bufferedMapBounds(176))) return
  const bounds = bufferedMapBounds(352)
  const token = ++detailRequestToken
  let files
  if (sourceFiles) {
    files = sourceFiles.filter(file => validLatLon(file.gps_latitude, file.gps_longitude)
      && bounds.contains([file.gps_latitude, file.gps_longitude]))
  } else {
    files = await getQueryFiles({
      ...props.queryParams,
      gpsMinLat: bounds.getSouth(), gpsMaxLat: bounds.getNorth(),
      gpsMinLon: bounds.getWest(), gpsMaxLon: bounds.getEast(),
    }, 0, MAP_DETAIL_LIMIT + 1)
  }
  if (token !== detailRequestToken || !map || !props.active || !files) return
  // Never render a truncated detail set as though it were the complete region.
  detailBounds = files.length <= MAP_DETAIL_LIMIT ? bounds : null
  visibleFiles = files.length <= MAP_DETAIL_LIMIT ? files : []
  renderMarkers()
}

async function getMapPoints() {
  sourceFiles = null
  if (props.querySource === 'collection' && props.collectionId) {
    const ids = await getCollectionQueryFileIds(props.collectionId, props.queryParams)
    sourceFiles = await getFilesByIds(ids || [])
  } else if (props.querySource === 'smart') {
    const ids = await getSmartQueryFileIds(props.queryParams)
    sourceFiles = await getFilesByIds(ids || [])
  } else if (props.querySource === 'search') {
    sourceFiles = await getFilesByIds(props.fileIds)
  } else {
    return getGpsMapPoints(props.queryParams)
  }
  return aggregateFiles(sourceFiles || [])
}

function aggregateFiles(files) {
  const cells = new Map()
  for (const file of files) {
    // Number(null) is 0, which would incorrectly place photos without GPS data
    // at the equator/prime meridian and include them in a map cluster.
    if (file.gps_latitude == null || file.gps_longitude == null || file.gps_latitude === '' || file.gps_longitude === '') continue
    const lat = Number(file.gps_latitude)
    const lon = Number(file.gps_longitude)
    if (!Number.isFinite(lat) || !Number.isFinite(lon)) continue
    const key = `${Math.round(lat * 100)}:${Math.round(lon * 100)}`
    const cell = cells.get(key) || { lat: 0, lon: 0, count: 0, file_id: Number(file.id) }
    cell.lat += lat
    cell.lon += lon
    cell.count++
    cell.file_id = Math.min(cell.file_id, Number(file.id))
    cells.set(key, cell)
  }
  return [...cells.values()].map(cell => ({ ...cell, lat: cell.lat / cell.count, lon: cell.lon / cell.count }))
}

function renderMarkers() {
  if (!map || !clusterWorker || !props.active || !isQueryMap.value) return
  // Invalidate pending results immediately, then combine zoomend/moveend.
  clusterRequestId++
  if (clusterRenderFrame !== null) return
  clusterRenderFrame = requestAnimationFrame(() => {
    clusterRenderFrame = null
    requestPhotoClusters()
  })
}

function requestPhotoClusters() {
  if (!map || !clusterWorker || !props.active || !isQueryMap.value) return
  const spacing = Math.round(Number(config.settings.mapMarkerSize || 64) * 0.75)
  const useDetails = zoom.value >= MAP_DETAIL_ZOOM && visibleFiles.length > 0
    && detailBounds?.contains(bufferedMapBounds(176))
  const source = useDetails ? 'details' : 'overview'
  const input = useDetails ? visibleFiles : points.value
  const previous = clusterSources.get(source)
  let cached = previous
  if (!cached || cached.input !== input || cached.maxZoom !== activeMaxZoom.value || cached.spacing !== spacing) {
    const candidates = useDetails
      ? visibleFiles.filter(file => validLatLon(file.gps_latitude, file.gps_longitude)).map(file => ({
        lat: Number(file.gps_latitude), lon: Number(file.gps_longitude), file_id: Number(file.id), count: 1,
        fileIds: [Number(file.id)],
      })).sort((a, b) => a.file_id - b.file_id)
      : points.value.map(point => ({ lat: point.lat, lon: point.lon, file_id: point.file_id, count: point.count, pointIds: [point.file_id] }))
    const signature = useDetails ? JSON.stringify(candidates) : null
    const unchanged = cached && cached.maxZoom === activeMaxZoom.value
      && cached.spacing === spacing && useDetails && cached.signature === signature
    cached = {
      input, signature, maxZoom: activeMaxZoom.value, spacing,
      version: unchanged ? cached.version : (cached?.version || 0) + 1,
      candidates,
    }
    clusterSources.set(source, cached)
  }
  const pixelBounds = map.getPixelBounds()
  clusterWorker.postMessage({
    requestId: clusterRequestId, source, version: cached.version,
    points: previous?.version === cached.version ? undefined : cached.candidates,
    maxZoom: cached.maxZoom, zoom: map.getZoom(), padding: useDetails ? 0.0001 : 0.01,
    spacing,
    bounds: { min: { x: pixelBounds.min.x, y: pixelBounds.min.y }, max: { x: pixelBounds.max.x, y: pixelBounds.max.y } },
  })
}

function applyPhotoClusters(clusters) {
  const nextMarkers = new Map()
  for (const cluster of clusters) {
    const key = JSON.stringify([cluster, config.settings.thumbnailSize, config.settings.mapMarkerSize])
    const marker = photoMarkers.get(key) || addPhotoMarker(cluster.lat, cluster.lon, cluster.file_id, cluster.count, cluster)
    nextMarkers.set(key, marker)
  }
  for (const [key, marker] of photoMarkers) {
    if (nextMarkers.has(key)) continue
    const element = marker.getElement()
    element?.classList.remove('map-photo-marker-visible')
    if (element) element.style.pointerEvents = 'none'
    marker.off()
    retiringMarkers.set(marker, setTimeout(() => {
      markerLayer?.removeLayer(marker)
      retiringMarkers.delete(marker)
    }, MAP_MARKER_FADE_MS))
  }
  photoMarkers = nextMarkers
}

// SQLite ROUND uses half-away-from-zero; local aggregateFiles uses Math.round.
function gpsCell(file, local) {
  const round = local ? Math.round : value => Math.sign(value) * Math.floor(Math.abs(value) + 0.5)
  return `${round(Number(file.gps_latitude) * 100)}:${round(Number(file.gps_longitude) * 100)}`
}

async function openPhotoCluster(cluster) {
  if (!map) return
  const requestId = ++clusterOpenRequestId
  const view = { lat: map.getCenter().lat, lon: map.getCenter().lng, zoom: map.getZoom() }
  let fileIds = cluster.fileIds
  if (!fileIds) {
    // Resolve only on click: keep full photo records out of the overview index.
    const localFiles = sourceFiles
    const queryParams = { ...props.queryParams }
    const ids = new Set(cluster.pointIds.map(Number))
    const representatives = localFiles
      ? localFiles.filter(file => ids.has(Number(file.id)))
      : await getFilesByIds([...ids])
    if (!representatives || requestId !== clusterOpenRequestId || !map || !props.active) return
    const cells = new Set(representatives.map(file => gpsCell(file, !!localFiles)))
    const files = localFiles || await getQueryFiles({
      ...queryParams,
      gpsMinLat: cluster.minLat, gpsMaxLat: cluster.maxLat,
      gpsMinLon: cluster.minLon, gpsMaxLon: cluster.maxLon,
    }, 0, 0)
    if (!files || requestId !== clusterOpenRequestId || !map || !props.active) return
    fileIds = files.filter(file => file.gps_latitude != null && file.gps_longitude != null
      && cells.has(gpsCell(file, !!localFiles))).map(file => Number(file.id))
  }
  if (requestId !== clusterOpenRequestId || !props.active) return
  emit('open-cluster', { ...cluster, fileIds, count: fileIds.length, view })
}

function addPhotoMarker(lat, lon, fileId, count, cluster = null) {
  if (lat == null || lon == null) return
  const size = Number(config.settings.mapMarkerSize || 64);
  const icon = L.divIcon({
    className: 'map-photo-marker-wrapper',
    iconSize: [size, size + 8],
    iconAnchor: [Math.round(size / 2), size + 8],
    html: `<div class="map-photo-marker"><img src="${getThumbUrl(fileId, false, config.settings.thumbnailSize || 512)}" />${count > 1 ? `<span>${count > 999 ? '999+' : count}</span>` : ''}</div>`,
  })
  const marker = L.marker([lat, lon], { icon, keyboard: false }).addTo(markerLayer)
  const element = marker.getElement()
  element?.style.setProperty('--map-photo-fade-duration', `${MAP_MARKER_FADE_MS}ms`)
  element?.style.setProperty('--map-photo-marker-size', `${size}px`)
  // Wait for the photo itself: otherwise its opacity animation can finish
  // while the thumbnail request is still pending.
  const show = () => {
    if (!map || retiringMarkers.has(marker) || !markerLayer.hasLayer(marker)) return
    const frame = requestAnimationFrame(() => {
      fadeFrames.delete(frame)
      const paintFrame = requestAnimationFrame(() => {
        fadeFrames.delete(paintFrame)
        if (!map || retiringMarkers.has(marker) || !markerLayer.hasLayer(marker)) return
        element?.classList.add('map-photo-marker-visible')
      })
      fadeFrames.add(paintFrame)
    })
    fadeFrames.add(frame)
  }
  const image = element?.querySelector('img')
  if (!image || image.complete) show()
  else {
    image.addEventListener('load', show, { once: true })
    image.addEventListener('error', show, { once: true })
  }
  marker.on('click', () => {
    if (!cluster || Number(count) === 1) {
      clusterOpenRequestId++
      emit('select-file', fileId)
      return
    }
    void openPhotoCluster(cluster)
  })
  marker.on('dblclick', (event) => {
    if (cluster && Number(count) !== 1) return
    L.DomEvent.stop(event.originalEvent)
    emit('preview-file', fileId)
  })
  return marker
}

function updateTheme() {
  const theme = getMapTheme(config.settings.mapProvider, config.settings.tiandituToken, config.settings.mapTheme)
  applyTheme(theme, false)
}

function applyTheme(theme, isFallback) {
  if (!map) return
  if (tileLayer) map.removeLayer(tileLayer)
  activeMaxZoom.value = theme.maxZoom
  map.setMaxZoom(theme.maxZoom)
  if (map.getZoom() > theme.maxZoom) map.setZoom(theme.maxZoom)
  if (!isFallback) tileErrorFallbackTriggered = false

  const created = createTileLayerGroup(L, theme)
  const activeLayer = created.layer
  tileLayer = activeLayer.addTo(map)
  created.tileLayers.forEach(layer => {
    layer.on('tileerror', () => {
      // Requests from a removed layer can still fail after a provider switch.
      // Ignore them so an old OSM request cannot replace the new provider.
      if (tileLayer !== activeLayer || tileErrorFallbackTriggered || isFallback) return
      tileErrorFallbackTriggered = true
      applyTheme(getGlobalMapTheme(config.settings.mapTheme), true)
    })
  })
}

function zoomIn() { if (map && zoom.value < activeMaxZoom.value) map.setZoom(zoom.value + 1) }
function zoomOut() { if (map && zoom.value > 0) map.setZoom(zoom.value - 1) }
function fitBounds() {
  if (!map || points.value.length === 0) return map?.setView([20, 0], 2)
  fitMapToPoints()
}
function fitMapToPoints() {
  map.fitBounds(L.latLngBounds(points.value.map(point => [point.lat, point.lon])), {
    padding: [20, 20],
    maxZoom: points.value.length === 1 ? Math.min(MAP_SINGLE_POINT_FIT_ZOOM, activeMaxZoom.value) : activeMaxZoom.value,
  })
}
function zoomCenter() {
  zoom.value = 13
  updateFromCoords()
}
function toggleMap() { config.settings.mapTheme = config.settings.mapTheme === 0 ? 1 : 0 }
function validLatLon(lat, lon) { return lat != null && lon != null && lat >= -90 && lat <= 90 && lon >= -180 && lon <= 180 }
async function openAppleMaps() {
  if (!showAppleMapsButton.value) return
  const label = props.label.trim() || 'Lap'
  await openExternalUrl(`maps://?ll=${props.lat},${props.lon}&q=${encodeURIComponent(label)}`)
}
function handleMapKeyDown(event) {
  const target = event.target
  if (target?.tagName === 'INPUT' || target?.tagName === 'TEXTAREA' || target?.isContentEditable) return
  if (!uiStore.mapActive || event.metaKey || event.ctrlKey || event.altKey) return
  if (event.key === '=') { event.preventDefault(); zoomIn() }
  if (event.key === '-') { event.preventDefault(); zoomOut() }
}
</script>
