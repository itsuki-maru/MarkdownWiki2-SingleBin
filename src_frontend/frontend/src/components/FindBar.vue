<script setup lang="ts">
import { nextTick, ref, watch } from 'vue';
import { usePageFind } from '@/utils/usePageFind';

const props = defineProps<{
    container: HTMLElement | null,
    showOpenInBrowser?: boolean,
}>();
const containerRef = ref<HTMLElement | null>(props.container);
watch(() => props.container, v => (containerRef.value = v));

const { query, countLabel, next, prev, clear } = usePageFind(
    containerRef, {
        normalizeJa: true,
        hotkey: "/",
        observeMutations: true,
    }
);

const inputRef = ref<HTMLInputElement | null>(null);

const focusInput = async () => {
    await nextTick();
    inputRef.value?.focus();
    inputRef.value?.select();
}

defineExpose({ focusInput });
</script>

<template>
    <div class="findbar">
        <input ref="inputRef" data-find-input v-model="query" placeholder="ページ内検索">
        <button type="button" @click="prev" aria-label="前へ">↑</button>
        <span class="count">{{ countLabel() }}</span>
        <button type="button" @click="next" aria-label="次へ">↓</button>
        <button type="button" @click="clear" aria-label="クリア">×</button>
    </div>
</template>

<style scoped>
.findbar {
    position: sticky;
    top: 0;
    z-index: 3;
    display: flex;
    gap: .5rem;
    padding: .5rem;
    background: var(--c-bg, #fff);
    border: 1px solid #898989;
    border-radius: 8px;
}

.findbar input {
    flex: 1;
    padding: .4rem .3rem;
}

.count {
    min-width: 3.0em;
    padding-top: 4px;
    text-align: center;
}

:global(mark[data-find]) { 
    background: #ffea00;
    padding: 0 .1em;
}
:global(mark[data-find].is-current) {
    outline: 3px solid #ff9800;
    background: #ff9800;
}
</style>