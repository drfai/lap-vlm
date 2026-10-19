import { createPhotoClusterCache } from './mapClusters'

const getClusters = createPhotoClusterCache()
self.onmessage = ({ data }) => {
  try {
    self.postMessage({ requestId: data.requestId, clusters: getClusters(data) })
  } catch (error) {
    self.postMessage({ requestId: data.requestId, error: String(error) })
  }
}
