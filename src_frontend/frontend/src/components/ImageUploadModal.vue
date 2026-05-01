<script setup lang="ts">
import { ref } from 'vue';
import type { UploadProgressState } from '@/interface';
import BaseModal from '@/components/BaseModal.vue';
import { useImageUpload } from '@/utils/useImageUpload';

const props = defineProps<{
  isOpen: boolean;
  isEditingMarker: boolean;
  isHttpsProtocol: boolean;
}>();

const emit = defineEmits<{
  close: [];
  uploaded: [markdownLink: string];
  message: [text: string];
  showUploadedUrl: [url: string, uniqueFileName: string];
  uploadProgressChange: [progress: UploadProgressState];
}>();

const emptyProgressState = (): UploadProgressState => ({
  isOpen: false,
  phase: 'preparing',
  percent: null,
  fileName: '',
  message: '',
});

const uploadProgress = ref<UploadProgressState>(emptyProgressState());
const {
  fileInputRef,
  selectedFileName,
  selectedFileSize,
  selectedMimeType,
  selectedFileTypeLabel,
  acceptedFileTypes,
  isUploadReady,
  isImageSendNow,
  formatFileSize,
  onImageSelect,
  imageFileSend,
  imageClear,
} = useImageUpload(
  (message) => emit('message', message),
  (markdownLink, uniqueFileName) => {
    if (props.isEditingMarker) {
      emit('uploaded', markdownLink);
      emit('message', '画像を挿入しました。');
      return;
    }

    if (props.isHttpsProtocol) {
      navigator.clipboard.writeText(markdownLink);
      emit('message', 'アップロード完了。リンクをクリップボードにコピーしました。');
      return;
    }

    emit('showUploadedUrl', markdownLink, uniqueFileName ?? '');
  },
  (progress) => {
    uploadProgress.value = progress;
    emit('uploadProgressChange', progress);
  },
  { showSuccessMessage: false },
);

const handleClose = (): void => {
  imageClear();
  uploadProgress.value = emptyProgressState();
  emit('uploadProgressChange', uploadProgress.value);
  emit('close');
};
</script>

<template>
  <BaseModal :isOpen="isOpen" @close="handleClose">
    <div class="upload-modal-content">
      <h2 class="modal-h2">画像・動画・PDFの追加</h2>
      <p class="upload-lead">
        追加したいファイルを 1 件選択してアップロードします。対応形式: {{ acceptedFileTypes }}
      </p>
      <p v-if="isEditingMarker" class="upload-context">
        アップロード後、マーカー編集欄へそのまま挿入できます。
      </p>

      <div class="upload-panel">
        <label for="image1" class="file-picker-card">
          <span class="file-picker-title">ファイルを選択</span>
          <span class="file-picker-subtitle">クリックして画像・動画・PDFを追加</span>
        </label>
        <input
          ref="fileInputRef"
          type="file"
          accept="image/jpeg,image/png,image/webp,image/gif,video/mp4,application/pdf"
          id="image1"
          class="file-input"
          @change="onImageSelect"
        />

        <div class="selection-summary" :class="{ empty: !selectedFileName }">
          <template v-if="selectedFileName">
            <div class="summary-row">
              <span class="summary-label">選択中</span>
              <span class="summary-value file-name">{{ selectedFileName }}</span>
            </div>
            <div class="summary-meta">
              <span class="meta-chip">{{ selectedFileTypeLabel }}</span>
              <span class="meta-chip">{{ formatFileSize(selectedFileSize) }}</span>
              <span v-if="selectedMimeType" class="meta-chip meta-chip-muted">{{
                selectedMimeType
              }}</span>
            </div>
          </template>
          <p v-else class="empty-text">
            まだファイルは選択されていません。30MB までアップロードできます。
          </p>
        </div>

        <div class="action-row">
          <button
            type="submit"
            class="btn-file-upload"
            :disabled="!isUploadReady"
            @click.prevent="imageFileSend()"
          >
            {{ isImageSendNow ? 'アップロード中...' : 'アップロード' }}
          </button>
          <button
            type="button"
            class="btn-secondary"
            :disabled="!selectedFileName"
            @click.prevent="imageClear()"
          >
            選択をクリア
          </button>
        </div>
      </div>
      <div class="btn-zone">
        <button @click.prevent="handleClose">閉じる</button>
      </div>
    </div>
  </BaseModal>
</template>

<style scoped>
.upload-modal-content {
  width: min(40vw, 560px);
}

.modal-h2 {
  border-bottom: solid 2px #acacac;
  text-align: center;
}

.upload-lead,
.upload-context {
  margin: 12px 0 0;
  line-height: 1.5;
}

.upload-context {
  color: #3559a9;
}

.upload-panel {
  margin-top: 18px;
  padding: 18px;
  border: 1px solid #d5dce8;
  border-radius: 16px;
  background: linear-gradient(180deg, #ffffff 0%, #f4f7fb 100%);
}

.file-input {
  display: none;
}

.file-picker-card {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 24px 18px;
  border: 2px dashed #7d96d7;
  border-radius: 14px;
  background-color: #f8fbff;
  color: #1d3776;
  text-align: center;
  cursor: pointer;
}

.file-picker-title {
  font-size: 17px;
  font-weight: 700;
}

.file-picker-subtitle {
  font-size: 13px;
  color: #49619b;
}

.selection-summary {
  margin-top: 14px;
  padding: 14px;
  border-radius: 14px;
  background-color: #ffffff;
  border: 1px solid #dbe3f0;
}

.selection-summary.empty {
  background-color: #f9fafc;
}

.summary-row {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.summary-label {
  font-size: 12px;
  color: #5d6b83;
}

.summary-value {
  color: #1e2430;
}

.file-name {
  font-weight: 700;
  word-break: break-all;
}

.summary-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 12px;
}

.meta-chip {
  padding: 4px 10px;
  border-radius: 999px;
  background-color: #e8eefb;
  color: #28407d;
  font-size: 12px;
}

.meta-chip-muted {
  background-color: #eef1f6;
  color: #556173;
}

.empty-text {
  margin: 0;
  color: #66758d;
  line-height: 1.5;
}

.action-row {
  display: flex;
  gap: 12px;
  margin-top: 18px;
}

.action-row button {
  flex: 1;
}

.btn-file-upload {
  min-height: 44px;
  font-size: 14px;
  background: rgb(28, 58, 190);
  color: #fff;
}

.btn-secondary {
  min-height: 44px;
  background: #d9deea;
  color: #25304a;
}

.btn-file-upload:disabled,
.btn-secondary:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.btn-zone {
  margin-top: 20px;
  display: flex;
  justify-content: flex-end;
}

@media (max-width: 900px) {
  .upload-modal-content {
    width: min(70vw, 560px);
  }
}

@media (max-width: 640px) {
  .upload-modal-content {
    width: min(90vw, 560px);
  }

  .action-row {
    flex-direction: column;
  }
}
</style>
