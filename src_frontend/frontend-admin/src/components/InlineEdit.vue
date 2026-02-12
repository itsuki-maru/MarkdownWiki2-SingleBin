<script setup lang="ts">
import { ref, nextTick } from 'vue';
import { useUsersStore } from '@/stores/users';

const userStore = useUsersStore();

const props = defineProps<{
  modelValue: string;
  dataKey: string;
}>();

const emit = defineEmits<{
  (event: 'update:modelValue', value: string, key?: string): void;
}>();

const isEditing = ref(false);
const editText = ref('');
const initText = ref('');
const editId = ref('');
const inputRef = ref<HTMLInputElement | null>(null);

const startEditing = async () => {
  editText.value = props.modelValue;
  initText.value = props.modelValue;
  editId.value = props.dataKey;
  isEditing.value = true;
  await nextTick();
  inputRef.value?.focus();
};

const saveEdit = () => {
  emit('update:modelValue', editText.value, props.dataKey);
  if (editText.value !== initText.value) {
    userStore.updatePublicName(editId.value, editText.value);
  }
  initText.value = '';
  isEditing.value = false;
};

const cancelEdit = () => {
  isEditing.value = false;
};
</script>

<template>
  <td v-on:click="startEditing" :data-key="props.dataKey">
    <template v-if="isEditing">
      <input
        ref="inputRef"
        v-model="editText"
        @blur="cancelEdit"
        @keyup.enter="saveEdit"
        @keyup.esc="cancelEdit"
        autofocus
        class="editing-input"
      />
    </template>
    <template v-else :id="editId">
      {{ modelValue }}
    </template>
  </td>
</template>

<style scoped>
.editing-input {
  border: 2px solid #007bff;
  background-color: #ffffff;
  padding: 4px;
  font-size: 1.2rem;
  width: 100%;
  text-align: center;
  box-sizing: border-box;
}
</style>
