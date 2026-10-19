// Distance-grid nearest-neighbor search, following Leaflet.markercluster:
// https://github.com/Leaflet/Leaflet.markercluster/blob/master/src/DistanceGrid.js
// Fixed photo anchors avoid centroid shifts introducing new icon collisions.
export function clusterPhotoPoints(points, project, padding = 0, spacing = 48) {
  // `spacing` is the minimum pixel distance between cluster anchors; it must
  // exceed half the marker diagonal to avoid overlapping icons.
  const cells = new Map()
  const clusters = []
  const sorted = [...points].sort((a, b) => b.count - a.count || Number(a.file_id) - Number(b.file_id) || a.lat - b.lat || a.lon - b.lon)
  for (const point of sorted) {
    const pixel = project(point)
    const x = Math.floor(pixel.x / spacing)
    const y = Math.floor(pixel.y / spacing)
    let nearest = null
    let distance = Infinity
    for (let dx = -1; dx <= 1; dx++) {
      for (let dy = -1; dy <= 1; dy++) {
        for (const candidate of cells.get(`${x + dx}:${y + dy}`) || []) {
          const px = Math.abs(candidate.pixel.x - pixel.x)
          const py = Math.abs(candidate.pixel.y - pixel.y)
          if (px * px + py * py < spacing * spacing && px * px + py * py < distance) {
            nearest = candidate.cluster
            distance = px * px + py * py
          }
        }
      }
    }
    const bounds = {
      minLat: point.minLat ?? point.lat - padding, maxLat: point.maxLat ?? point.lat + padding,
      minLon: point.minLon ?? point.lon - padding, maxLon: point.maxLon ?? point.lon + padding,
    }
    if (nearest) {
      nearest.count += point.count
      if (nearest.fileIds && point.fileIds) nearest.fileIds.push(...point.fileIds)
      if (nearest.pointIds && point.pointIds) nearest.pointIds.push(...point.pointIds)
      nearest.minLat = Math.min(nearest.minLat, bounds.minLat)
      nearest.maxLat = Math.max(nearest.maxLat, bounds.maxLat)
      nearest.minLon = Math.min(nearest.minLon, bounds.minLon)
      nearest.maxLon = Math.max(nearest.maxLon, bounds.maxLon)
    } else {
      const cluster = { ...point, ...bounds, ...(point.fileIds ? { fileIds: [...point.fileIds] } : {}), ...(point.pointIds ? { pointIds: [...point.pointIds] } : {}) }
      clusters.push(cluster)
      const key = `${x}:${y}`
      if (!cells.has(key)) cells.set(key, [])
      cells.get(key).push({ pixel, cluster })
    }
  }
  return clusters
}

// Build parents from the preceding zoom's clusters, as in Supercluster:
// https://github.com/mapbox/supercluster/blob/main/index.js
// Keep all input points in the index; viewport culling happens only at rendering.
export function buildPhotoClusterIndex(points, project, maxZoom, padding = 0, spacing = 48) {
  const levels = new Map()
  let children = points
  for (let zoom = maxZoom; zoom >= 0; zoom--) {
    children = clusterPhotoPoints(children, point => project(point, zoom), padding, spacing)
    levels.set(zoom, children)
  }
  return levels
}

// Leaflet EPSG:3857 / 256px tile projection, independent of DOM for workers.
export function projectPhotoPoint(point, zoom) {
  const scale = 256 * 2 ** zoom
  const lat = Math.max(-85.0511287798, Math.min(85.0511287798, point.lat))
  const sin = Math.sin(lat * Math.PI / 180)
  return {
    x: scale * (point.lon / 360 + 0.5),
    y: scale * (0.5 - Math.log((1 + sin) / (1 - sin)) / (4 * Math.PI)),
  }
}

export function photoClusterInView(cluster, zoom, bounds) {
  const pixel = projectPhotoPoint(cluster, zoom)
  // Hierarchical anchor displacement is < 96px; include the 72px icon too.
  const buffer = 176
  return pixel.x >= bounds.min.x - buffer && pixel.x <= bounds.max.x + buffer
    && pixel.y >= bounds.min.y - buffer && pixel.y <= bounds.max.y + buffer
}

export function createPhotoClusterCache() {
  const sources = new Map()
  return ({ source, version, points, maxZoom, zoom, padding, spacing, bounds }) => {
    let cached = sources.get(source)
    if (!cached || cached.version !== version || cached.maxZoom !== maxZoom || cached.spacing !== spacing) {
      cached = { version, maxZoom, spacing, levels: buildPhotoClusterIndex(points, projectPhotoPoint, maxZoom, padding, spacing) }
      sources.set(source, cached)
    }
    return (cached.levels.get(Math.floor(zoom)) || []).filter(cluster => photoClusterInView(cluster, zoom, bounds))
  }
}
