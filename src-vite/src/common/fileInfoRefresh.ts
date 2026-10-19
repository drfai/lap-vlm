import { ref } from 'vue';

// Active metadata sidebars reload once after a batch; inactive ones reload on entry.
export const fileInfoRevision = ref(0);
