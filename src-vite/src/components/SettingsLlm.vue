<template>
  <div class="flex flex-col items-start justify-start gap-4 h-full text-base-content/70 cursor-default overflow-y-auto">

    <div class="w-full max-w-lg rounded-box border border-base-content/5 bg-base-300/30 p-4 shadow-sm">
      <div class="flex flex-row items-center justify-between">
        <div class="flex flex-col">
          <h4 class="text-base">{{ $t('settings.llm.title') }}</h4>
          <p class="mt-1 text-xs text-base-content/40">
            {{ $t('settings.llm.subtitle') }}
          </p>
        </div>
        <input
          type="checkbox"
          class="toggle toggle-primary toggle-sm"
          :checked="form.enabled"
          @change="form.enabled = $event.target.checked"
        />
      </div>
    </div>

    <!-- 连接配置 -->
    <div class="w-full max-w-lg rounded-box border border-base-content/5 bg-base-300/30 p-4 shadow-sm space-y-3">
      <div class="text-sm">{{ $t('settings.llm.connection') }}</div>

      <div class="grid grid-cols-[110px_1fr] items-center gap-3">
        <label class="text-xs text-base-content/40">{{ $t('settings.llm.base_url') }}</label>
        <input
          v-model="form.baseUrl"
          type="text"
          placeholder="http://127.0.0.1:1234"
          class="input input-bordered input-sm w-full"
          :disabled="!form.enabled"
        />
      </div>

      <div class="grid grid-cols-[110px_1fr] items-center gap-3">
        <label class="text-xs text-base-content/40">{{ $t('settings.llm.model') }}</label>
        <input
          v-model="form.model"
          type="text"
          placeholder="qwen3-vl-8b-instruct"
          class="input input-bordered input-sm w-full"
          :disabled="!form.enabled"
        />
      </div>

      <div class="grid grid-cols-[110px_1fr] items-center gap-3">
        <label class="text-xs text-base-content/40">{{ $t('settings.llm.api_key') }}</label>
        <input
          v-model="form.apiKey"
          type="password"
          placeholder="—"
          class="input input-bordered input-sm w-full"
          :disabled="!form.enabled"
        />
      </div>

      <div class="flex flex-row items-center gap-2 pt-1">
        <button class="btn btn-sm btn-ghost" :disabled="testing || !form.enabled" @click="onTest">
          <span v-if="testing" class="loading loading-spinner loading-xs"></span>
          {{ $t('settings.llm.test') }}
        </button>
        <span v-if="testMsg" :class="['text-xs', testOk ? 'text-success' : 'text-error']">{{ testMsg }}</span>
      </div>
    </div>

    <!-- 索引进度 -->
    <div class="w-full max-w-lg rounded-box border border-base-content/5 bg-base-300/30 p-4 shadow-sm space-y-3">
      <div class="text-sm">{{ $t('settings.llm.index') }}</div>

      <div v-if="stats" class="grid grid-cols-3 gap-2 text-center">
        <div class="rounded bg-base-100/40 py-2">
          <div class="text-lg leading-none">{{ stats.indexed }}</div>
          <div class="mt-1 text-[10px] text-base-content/40">{{ $t('settings.llm.indexed') }}</div>
        </div>
        <div class="rounded bg-base-100/40 py-2">
          <div class="text-lg leading-none">{{ stats.pending }}</div>
          <div class="mt-1 text-[10px] text-base-content/40">{{ $t('settings.llm.pending') }}</div>
        </div>
        <div class="rounded bg-base-100/40 py-2">
          <div class="text-lg leading-none">{{ stats.total }}</div>
          <div class="mt-1 text-[10px] text-base-content/40">{{ $t('settings.llm.total') }}</div>
        </div>
      </div>

      <div v-if="progress && progress.phase !== 'idle'" class="space-y-1">
        <progress
          class="progress progress-primary w-full h-1.5"
          :value="progress.current"
          :max="Math.max(progress.total, 1)"
        ></progress>
        <div class="flex justify-between text-[10px] text-base-content/40">
          <span>{{ $t(`settings.llm.phase.${progress.phase}`) }}</span>
          <span>
            {{ progress.current }} / {{ progress.total }}
            <template v-if="progress.failed"> · {{ $t('settings.llm.failed') }} {{ progress.failed }}</template>
          </span>
        </div>
      </div>

      <div class="flex flex-row items-center gap-2">
        <button
          v-if="!indexing"
          class="btn btn-sm btn-primary"
          :disabled="!form.enabled || saving"
          @click="onStart"
        >
          {{ $t('settings.llm.start') }}
        </button>
        <button v-else class="btn btn-sm btn-warning" @click="onCancel">
          {{ $t('settings.llm.cancel') }}
        </button>
        <button class="btn btn-sm btn-ghost" :disabled="indexing" @click="refreshStats">
          {{ $t('settings.llm.refresh') }}
        </button>
      </div>
    </div>

    <!-- 高级选项 -->
    <div class="w-full max-w-lg rounded-box border border-base-content/5 bg-base-300/30 p-4 shadow-sm space-y-3">
      <div class="text-sm">{{ $t('settings.llm.advanced') }}</div>

      <div class="grid grid-cols-[110px_1fr] items-center gap-3">
        <label class="text-xs text-base-content/40">{{ $t('settings.llm.preview_max_px') }}</label>
        <div class="flex items-center gap-2">
          <input
            v-model.number="form.previewMaxPx"
            type="range" min="256" max="1280" step="64"
            class="range range-xs w-32"
            :disabled="!form.enabled"
          />
          <span class="text-xs w-12 text-right">{{ form.previewMaxPx }}</span>
        </div>
      </div>

      <div class="grid grid-cols-[110px_1fr] items-center gap-3">
        <label class="text-xs text-base-content/40">{{ $t('settings.llm.max_tokens') }}</label>
        <div class="flex items-center gap-2">
          <input
            v-model.number="form.maxTokens"
            type="range" min="256" max="4096" step="256"
            class="range range-xs w-32"
            :disabled="!form.enabled"
          />
          <span class="text-xs w-12 text-right">{{ form.maxTokens }}</span>
        </div>
      </div>

      <div class="grid grid-cols-[110px_1fr] items-center gap-3">
        <label class="text-xs text-base-content/40">{{ $t('settings.llm.concurrency') }}</label>
        <div class="flex items-center gap-2">
          <input
            v-model.number="form.concurrency"
            type="range" min="1" max="4" step="1"
            class="range range-xs w-32"
            :disabled="!form.enabled"
          />
          <span class="text-xs w-12 text-right">{{ form.concurrency }}</span>
        </div>
      </div>

      <label class="flex items-center gap-2 text-xs">
        <input v-model="form.disableThinking" type="checkbox" class="checkbox checkbox-xs" :disabled="!form.enabled" />
        <span>{{ $t('settings.llm.disable_thinking') }}</span>
      </label>
      <p class="text-[10px] leading-relaxed text-base-content/30">
        {{ $t('settings.llm.disable_thinking_hint') }}
      </p>
    </div>

    <!-- FORK: 索引字段编辑器 -->
    <div class="w-full max-w-lg rounded-box border border-base-content/5 bg-base-300/30 p-4 shadow-sm space-y-3">
      <div class="text-sm">{{ $t('settings.llm.fields') }}</div>
      <p class="text-[10px] leading-relaxed text-base-content/30">
        {{ $t('settings.llm.fields_hint') }}
      </p>

      <!-- 内置字段（key 锁定，只改描述） -->
      <div class="text-xs text-base-content/40 pt-1">{{ $t('settings.llm.builtin_fields') }}</div>
      <div v-for="f in builtinFields" :key="f.key" class="grid grid-cols-[104px_1fr] items-center gap-2">
        <label class="text-xs text-base-content/40 font-mono" :title="$t('settings.llm.key_locked')">{{ f.key }}</label>
        <input
          :value="builtinDesc(f.key)"
          type="text"
          class="input input-bordered input-xs w-full"
          :placeholder="f.desc"
          :disabled="!form.enabled"
          @input="setBuiltinDesc(f.key, $event.target.value)"
        />
      </div>

      <!-- 自定义字段（key + 描述 + 类型全可改） -->
      <div class="flex items-center justify-between pt-2">
        <div class="text-xs text-base-content/40">{{ $t('settings.llm.custom_fields') }}</div>
        <button class="btn btn-xs btn-ghost" :disabled="!form.enabled" @click="addCustomField">
          + {{ $t('settings.llm.add_field') }}
        </button>
      </div>

      <div v-if="!form.promptFields.customFields.length" class="text-[10px] text-base-content/30 py-1">
        {{ $t('settings.llm.no_custom_fields') }}
      </div>

      <div
        v-for="(cf, i) in form.promptFields.customFields"
        :key="i"
        class="rounded border border-base-content/5 bg-base-100/30 p-2 space-y-2"
      >
        <div class="flex items-center gap-2">
          <input
            v-model="cf.key"
            type="text"
            class="input input-bordered input-xs w-28 font-mono"
            :class="{ 'input-error': fieldKeyError(cf, i) }"
            placeholder="colors"
            :disabled="!form.enabled"
          />
          <select v-model="cf.kind" class="select select-bordered select-xs w-20" :disabled="!form.enabled">
            <option value="string">{{ $t('settings.llm.type_string') }}</option>
            <option value="array">{{ $t('settings.llm.type_array') }}</option>
            <option value="number">{{ $t('settings.llm.type_number') }}</option>
          </select>
          <button class="btn btn-xs btn-ghost text-error ml-auto" :disabled="!form.enabled" @click="removeCustomField(i)">
            {{ $t('settings.llm.remove') }}
          </button>
        </div>
        <input
          v-model="cf.desc"
          type="text"
          class="input input-bordered input-xs w-full"
          :class="{ 'input-error': !cf.desc.trim() }"
          :placeholder="$t('settings.llm.field_desc')"
          :disabled="!form.enabled"
        />
        <div v-if="fieldKeyError(cf, i)" class="text-[10px] text-error">
          {{ $t('settings.llm.key_reserved') }}
        </div>
        <div v-else-if="!cf.desc.trim()" class="text-[10px] text-error">
          {{ $t('settings.llm.desc_empty') }}
        </div>
      </div>
    </div>

    <!-- 保存 -->
    <div class="w-full max-w-lg flex flex-row items-center justify-end gap-2 pb-2">
      <span v-if="savedAt" class="text-xs text-success">{{ $t('settings.llm.saved') }}</span>
      <button class="btn btn-sm btn-primary" :disabled="saving" @click="onSave">
        <span v-if="saving" class="loading loading-spinner loading-xs"></span>
        {{ $t('common.save') }}
      </button>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import {
  getLlmConfig,
  saveLlmConfig,
  testLlmConnection,
  startLlmIndexing,
  cancelLlmIndexing,
  getLlmIndexStats,
  listenLlmIndexProgress,
  listenLlmIndexFinished,
} from '../common/api.js';

const { t } = useI18n();

const DEFAULT_FORM = {
  enabled: false,
  baseUrl: 'http://127.0.0.1:1234',
  model: '',
  apiKey: '',
  disableThinking: true,
  maxTokens: 1024,
  temperature: 0.1,
  previewMaxPx: 768,
  concurrency: 1,
  timeoutSecs: 300,
  retryOnParseError: 1,
  albumIds: [],
  // FORK: 索引字段定义（内置描述可改 + 自定义字段可增删）
  promptFields: { builtinDescs: {}, customFields: [] },
  enableRerank: false,
  rerankTopN: 30,
};

const form = reactive({ ...DEFAULT_FORM });
const stats = ref(null);
const progress = ref(null);
const indexing = ref(false);
const testing = ref(false);
const saving = ref(false);
const testMsg = ref('');
const testOk = ref(false);
const savedAt = ref(false);

// FORK: 内置字段的 key 与默认描述（key 与 DB 列绑定，不可改）
// 描述默认用中文——与 prompt 模板的「所有字段用简体中文填写」一致。
// 用户可以自由改写这些描述（中英文均可），改写后保存即生效。
const BUILTIN_FIELDS = [
  { key: 'caption', desc: '一到两句客观描述：画面里有什么、在做什么、什么环境' },
  { key: 'objects', desc: '画面中可见的主要物体或人，逐项列出' },
  { key: 'scene', desc: '场景类型，如：室内、户外、街道、海边、山地、餐厅、办公室' },
  { key: 'action', desc: '画面中正在发生的动作；若无明确动作填 null' },
  { key: 'relations', desc: '物体或人之间的空间关系，如：杯子放在桌上、人坐在椅子上' },
  { key: 'text', desc: '画面中可读的文字；若无填 null' },
  { key: 'people_count', desc: '画面中可分辨的人脸数量，数字，看不清填 0' },
];

const builtinFields = BUILTIN_FIELDS;

function builtinDesc(key) {
  const v = form.promptFields?.builtinDescs?.[key];
  return (v && v.trim()) ? v : '';
}

function setBuiltinDesc(key, val) {
  if (!form.promptFields.builtinDescs) form.promptFields.builtinDescs = {};
  form.promptFields.builtinDescs[key] = val;
}

function addCustomField() {
  form.promptFields.customFields.push({ key: '', desc: '', kind: 'string' });
}

function removeCustomField(i) {
  form.promptFields.customFields.splice(i, 1);
}

const KEY_RE = /^[A-Za-z][A-Za-z0-9_]*$/;

function fieldKeyError(cf, i) {
  const k = (cf.key || '').trim();
  if (!k) return false; // 空 key 由 desc 检查提示，不重复报错
  if (!KEY_RE.test(k)) return true;
  if (BUILTIN_FIELDS.some((f) => f.key === k)) return true;
  // 与其他自定义字段重名
  return form.promptFields.customFields.some((o, j) => j !== i && (o.key || '').trim() === k);
}

let unlistenProgress = null;
let unlistenFinished = null;

async function refreshStats() {
  stats.value = await getLlmIndexStats();
}

async function onTest() {
  // 先保存，让后端拿到最新连接参数
  await onSave();
  testing.value = true;
  testMsg.value = '';
  try {
    const msg = await testLlmConnection();
    testOk.value = true;
    testMsg.value = msg || t('settings.llm.test_ok');
  } catch (e) {
    testOk.value = false;
    testMsg.value = typeof e === 'string' ? e : (e?.message || t('settings.llm.test_fail'));
  } finally {
    testing.value = false;
  }
}

async function onStart() {
  await onSave();
  indexing.value = true;
  testMsg.value = '';
  try {
    await startLlmIndexing();
  } catch (e) {
    const msg = typeof e === 'string' ? e : (e?.message || t('settings.llm.start_fail'));
    // FORK: "already running" 表示索引其实在后台跑（可能是上次启动后 UI 状态丢了）。
    // 这时必须保持 indexing=true，让取消按钮可见——否则用户没有任何停止手段。
    if (/already running/i.test(msg)) {
      testOk.value = false;
      testMsg.value = msg + '（可直接点「取消」停止）';
    } else {
      indexing.value = false;
      testOk.value = false;
      testMsg.value = msg;
    }
  }
}

async function onCancel() {
  await cancelLlmIndexing();
}

async function onSave() {
  saving.value = true;
  try {
    const out = { ...form };
    // FORK: 清理无效的自定义字段（空/非法 key、空描述、重名），
    // 否则后端 build_index_prompt 会拼出坏 prompt。
    const seen = new Set(BUILTIN_FIELDS.map((f) => f.key));
    out.promptFields = {
      builtinDescs: { ...(form.promptFields?.builtinDescs || {}) },
      customFields: (form.promptFields?.customFields || []).filter((cf) => {
        const k = (cf.key || '').trim();
        const d = (cf.desc || '').trim();
        if (!k || !d || !KEY_RE.test(k) || seen.has(k)) return false;
        seen.add(k);
        return true;
      }).map((cf) => ({ key: cf.key.trim(), desc: cf.desc.trim(), kind: cf.kind || 'string' })),
    };
    await saveLlmConfig(out);
    savedAt.value = true;
    setTimeout(() => (savedAt.value = false), 2000);
  } finally {
    saving.value = false;
  }
}

onMounted(async () => {
  const cfg = await getLlmConfig();
  if (cfg) {
    Object.assign(form, DEFAULT_FORM, cfg);
    // 兼容旧配置：promptFields 可能缺失或字段不全
    if (!form.promptFields || typeof form.promptFields !== 'object') {
      form.promptFields = { builtinDescs: {}, customFields: [] };
    } else {
      if (!form.promptFields.builtinDescs) form.promptFields.builtinDescs = {};
      if (!Array.isArray(form.promptFields.customFields)) form.promptFields.customFields = [];
    }
  }
  await refreshStats();

  unlistenProgress = await listenLlmIndexProgress((e) => {
    progress.value = e.payload;
    // FORK: 失败原因实时显示在按钮行的消息位（之前后端只写 stderr，前端完全看不到）
    if (e.payload?.lastError) {
      testOk.value = false;
      testMsg.value = e.payload.lastError;
    }
    if (e.payload?.total) {
      // FORK: 三个数字的口径（与 get_stats / refreshStats 保持一致）：
      //   indexed = 真实已索引（progress.succeeded 带基线，含本次成功）
      //   pending = 后端算的真实待处理（progress.pending）
      //   total   = 库内可索引总数（= indexed + pending，与"总计"卡片一致）
      // progress.total 是"本次任务量"（进度条分母），不能直接用在卡片上，
      // 所以用 indexed + pending 还原总数。
      const indexed = e.payload.succeeded || 0;
      const pending = e.payload.pending ?? 0;
      stats.value = {
        indexed,
        pending,
        total: indexed + pending,
      };
    }
  });

  unlistenFinished = await listenLlmIndexFinished((e) => {
    indexing.value = false;
    progress.value = { phase: 'idle', current: 0, total: 0, succeeded: 0, failed: 0, pending: 0 };
    const p = e.payload;
    if (p && p.error) {
      testOk.value = false;
      testMsg.value = p.error;
    } else if (p && p.total !== undefined) {
      testOk.value = true;
      // FORK: total = 本次任务量，succeeded = 真实累计（含基线），两者不同口径。
      // 本次成果用 succeededThisRun；同时显示库内真实累计，便于确认没丢进度。
      const thisRun = p.succeededThisRun ?? p.succeeded ?? 0;
      const realTotal = p.succeeded ?? 0;
      testMsg.value = `done: ${thisRun}/${p.total}${p.failed ? `, failed ${p.failed}` : ''}` +
        (realTotal !== thisRun ? ` · 累计已索引 ${realTotal}` : '');
    }
    refreshStats();
  });
});

onUnmounted(() => {
  if (unlistenProgress) unlistenProgress();
  if (unlistenFinished) unlistenFinished();
});
</script>
