const GLOBAL_THEMES = [
  {
    maxZoom: 19,
    layers: [
      { url: 'https://tile.openstreetmap.org/{z}/{x}/{y}.png', attribution: 'OpenStreetMap' },
    ],
  },
  {
    maxZoom: 17,
    layers: [
      { url: 'https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}', attribution: 'Powered by Esri' },
    ],
  },
];

function tiandituUrl(layer, token) {
  return `https://t{s}.tianditu.gov.cn/${layer}_w/wmts?SERVICE=WMTS&REQUEST=GetTile&VERSION=1.0.0&LAYER=${layer.slice(0, 3)}&STYLE=default&TILEMATRIXSET=w&TILEMATRIX={z}&TILEROW={y}&TILECOL={x}&FORMAT=tiles&tk=${encodeURIComponent(token)}`;
}

export function getMapThemes(provider, tiandituToken) {
  const token = String(tiandituToken || '').trim();
  if (provider !== 'tianditu' || !token) return GLOBAL_THEMES;

  const attribution = 'Tianditu';
  return [
    {
      maxZoom: 18,
      layers: [
        { url: tiandituUrl('vec', token), attribution, subdomains: '01234567' },
        { url: tiandituUrl('cva', token), attribution, subdomains: '01234567' },
      ],
    },
    {
      maxZoom: 18,
      layers: [
        { url: tiandituUrl('img', token), attribution, subdomains: '01234567' },
        { url: tiandituUrl('cia', token), attribution, subdomains: '01234567' },
      ],
    },
  ];
}

export function getMapTheme(provider, tiandituToken, themeIndex) {
  const themes = getMapThemes(provider, tiandituToken);
  return themes[Number(themeIndex) === 1 ? 1 : 0];
}

export function getGlobalMapTheme(themeIndex) {
  return GLOBAL_THEMES[Number(themeIndex) === 1 ? 1 : 0];
}

export function createTileLayerGroup(L, theme) {
  const layers = theme.layers.map(({ url, attribution, subdomains }) => L.tileLayer(url, {
    attribution,
    maxZoom: theme.maxZoom,
    ...(subdomains ? { subdomains } : {}),
  }));
  return {
    layer: layers.length === 1 ? layers[0] : L.layerGroup(layers),
    tileLayers: layers,
  };
}
