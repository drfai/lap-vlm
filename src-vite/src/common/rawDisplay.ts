import { useConfigStore } from '@/stores/configStore';

export type RawDisplayOptions = { mode: 'embedded' | 'rendered'; autoBright: boolean; preferPair: boolean };
export function getRawDisplayOptions(): RawDisplayOptions {
  const settings = useConfigStore().settings;
  return {
    mode: settings.rawPreviewSource,
    autoBright: settings.rawRenderBrightness === 'brightened',
    preferPair: settings.groupRawJpegPairs && settings.rawPairDisplaySource === 'jpeg',
  };
}

export function rawDisplayKey() {
  const options = getRawDisplayOptions();
  return `${options.mode}:${options.autoBright}:${options.preferPair}`;
}

export function appendRawDisplayParams(params: URLSearchParams, options = getRawDisplayOptions()) {
  params.set('rawPreviewMode', options.mode);
  params.set('rawAutoBright', String(options.autoBright));
  params.set('rawPairDisplay', options.preferPair ? 'jpeg' : 'raw');
}

export type RawPreviewSource = 'embedded' | 'rendered' | 'brightened' | 'pair' | 'image' | '';
export function nextRawPreviewMode(current: RawPreviewSource, unavailable: boolean, defaults: RawDisplayOptions): Exclude<RawPreviewSource, 'pair' | 'image' | ''> {
  const modes: Exclude<RawPreviewSource, 'pair' | 'image' | ''>[] = unavailable ? ['rendered', 'brightened'] : ['embedded', 'rendered', 'brightened'];
  if (current === 'pair' || current === 'image' || !current) {
    return defaults.mode === 'embedded' && !unavailable ? 'embedded' : defaults.autoBright ? 'brightened' : 'rendered';
  }
  return modes[(modes.indexOf(current) + 1) % modes.length];
}
