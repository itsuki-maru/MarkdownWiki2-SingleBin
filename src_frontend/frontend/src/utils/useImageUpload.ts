import { computed, ref } from 'vue';
import type { AxiosProgressEvent } from 'axios';
import { AxiosError } from 'axios';
import type { ImageData, UploadProgressState } from '@/interface';
import { imageUploadUrl } from '@/router/urls';
import { baseUrl } from '@/setting';
import { useImageStore } from '@/stores/images';
import { useImageResize } from '@/utils/useImageResize';
import apiClient from '@/axiosClient';
import { isPDF, isMP4 } from '@/utils/markedSetup';

type AssetKind = 'image' | 'video' | 'pdf' | '';

const ALLOWED_MIME_TYPES = [
  'image/jpeg',
  'image/png',
  'image/webp',
  'image/gif',
  'video/mp4',
  'application/pdf',
];
const MAX_UPLOAD_FILE_SIZE = 30 * 1024 * 1024;

type ImageUploadOptions = {
  showSuccessMessage?: boolean;
  successMessage?: string;
};

export function useImageUpload(
  onMessage: (msg: string) => void,
  onUploadSuccess: (markdownStr: string, uniqueFileName?: string) => void,
  onUploadProgressChange: (progress: UploadProgressState) => void = () => {},
  options: ImageUploadOptions = {},
) {
  const imageStore = useImageStore();
  const { resizeImageWithCanvas } = useImageResize();
  const selectedImageBlob = ref<Blob | null>(null);
  const selectedFileName = ref<string>('');
  const selectedFileSize = ref<number | null>(null);
  const selectedMimeType = ref<string>('');
  const selectedAssetKind = ref<AssetKind>('');
  const isImageSendNow = ref(false);
  const fileInputRef = ref<HTMLInputElement | null>(null);

  const acceptedFileTypes = 'JPEG, PNG, WebP, GIF, MP4, PDF';

  const emptyProgressState = (): UploadProgressState => ({
    isOpen: false,
    phase: 'preparing',
    percent: null,
    fileName: '',
    message: '',
  });

  const emitProgress = (progress: UploadProgressState): void => {
    onUploadProgressChange(progress);
  };

  const handleUploadProgress = (progressEvent: AxiosProgressEvent): void => {
    const loaded = progressEvent.loaded ?? 0;
    const total = progressEvent.total ?? undefined;
    const percent = total && total > 0 ? Math.min(100, Math.round((loaded / total) * 100)) : null;

    emitProgress({
      isOpen: true,
      phase: percent === 100 ? 'finalizing' : 'uploading',
      percent: percent === 100 ? null : percent,
      fileName: selectedFileName.value,
      message:
        percent === 100
          ? 'アップロード完了。サーバーで保存処理中です。'
          : 'ファイルをアップロードしています。',
      loadedBytes: loaded,
      totalBytes: total,
    });
  };

  const selectedFileTypeLabel = computed(() => {
    switch (selectedAssetKind.value) {
      case 'image':
        return '画像';
      case 'video':
        return '動画';
      case 'pdf':
        return 'PDF';
      default:
        return '未選択';
    }
  });

  const isUploadReady = computed(() => selectedImageBlob.value !== null && !isImageSendNow.value);

  const formatFileSize = (size: number | null): string => {
    if (size === null) return '-';
    if (size < 1024 * 1024) return `${Math.round(size / 1024)} KB`;
    return `${(size / (1024 * 1024)).toFixed(1)} MB`;
  };

  const resolveAssetKind = (file: File): AssetKind => {
    if (file.type.startsWith('image/')) return 'image';
    if (file.type === 'video/mp4') return 'video';
    if (file.type === 'application/pdf') return 'pdf';
    return '';
  };

  const onImageSelect = async (): Promise<void> => {
    const element = fileInputRef.value;
    if (!element || element.value === '' || element.value === null) {
      onMessage('画像ファイルを選択してください。');
      return;
    }

    const fileObj = element.files?.[0];
    if (!fileObj) {
      return;
    }

    selectedFileName.value = fileObj.name;
    selectedFileSize.value = fileObj.size;
    selectedMimeType.value = fileObj.type;
    selectedAssetKind.value = resolveAssetKind(fileObj);

    if (!ALLOWED_MIME_TYPES.includes(fileObj.type)) {
      onMessage('許可されていない形式のファイルです。');
      imageClear();
      return;
    }

    if (fileObj.size > MAX_UPLOAD_FILE_SIZE) {
      onMessage('30MBを超えるファイルはアップロードできません。');
      imageClear();
      return;
    }

    if (fileObj.type.startsWith('image/')) {
      try {
        emitProgress({
          isOpen: true,
          phase: 'preparing',
          percent: null,
          fileName: fileObj.name,
          message: '画像をアップロード用に最適化しています。',
        });
        selectedImageBlob.value = await resizeImageWithCanvas(fileObj);
      } catch (error) {
        console.error('リサイズエラー: ', error);
        selectedImageBlob.value = null;
      } finally {
        emitProgress(emptyProgressState());
      }
    } else {
      selectedImageBlob.value = fileObj;
    }
  };

  const imageFileSend = async (): Promise<void> => {
    if (isImageSendNow.value === true) {
      return;
    } else {
      isImageSendNow.value = true;
    }

    if (!selectedImageBlob.value) {
      onMessage('ファイルを選択してください。');
      isImageSendNow.value = false;
      emitProgress(emptyProgressState());
      return;
    }

    const payload = new FormData();
    payload.append('upload_file', selectedImageBlob.value, selectedFileName.value);

    try {
      emitProgress({
        isOpen: true,
        phase: 'uploading',
        percent: 0,
        fileName: selectedFileName.value,
        message: 'ファイルをアップロードしています。',
        loadedBytes: 0,
      });
      const response = await apiClient.post(imageUploadUrl, payload, {
        onUploadProgress: handleUploadProgress,
      });

      const newImageData: ImageData = {
        id: response.data['new_image_id'],
        user_id: response.data['user_id'],
        filename: response.data['filename'],
        uuid_filename: response.data['uuid_filename'],
      };
      imageStore.addImage(newImageData);

      const uniqueFileName = response.data['uuid_filename'];

      let imageUrlMarkdown = '';
      if (isMP4(uniqueFileName)) {
        imageUrlMarkdown = `?[${selectedFileName.value}](${baseUrl}/static/images/${uniqueFileName})`;
      } else {
        if (isPDF(uniqueFileName)) {
          imageUrlMarkdown = `[${selectedFileName.value}](${baseUrl}/static/images/${uniqueFileName})`;
        } else {
          imageUrlMarkdown = `![${selectedFileName.value}](${baseUrl}/static/images/${uniqueFileName})`;
        }
      }

      imageStore.initList();
      if (options.showSuccessMessage !== false) {
        onMessage(options.successMessage ?? 'アップロード完了。コンテンツを挿入しました。');
      }
      onUploadSuccess(imageUrlMarkdown, uniqueFileName);
      imageClear();
      return;
    } catch (error) {
      if (apiClient.isAxiosError(error)) {
        const axiosError = error as AxiosError;
        if (axiosError.response) {
          onMessage(`${axiosError.response.data}`);
          console.error(`Upload error: ${axiosError.response.status}`, axiosError.response.data);
        }
      } else {
        console.error(error);
        onMessage('アップロードに失敗しました。ファイルサイズやファイルの種類を確認してください。');
      }
    } finally {
      selectedImageBlob.value = null;
      selectedFileName.value = '';
      selectedFileSize.value = null;
      selectedMimeType.value = '';
      selectedAssetKind.value = '';
      isImageSendNow.value = false;
      emitProgress(emptyProgressState());
    }
  };

  const imageClear = (): void => {
    selectedFileName.value = '';
    selectedFileSize.value = null;
    selectedMimeType.value = '';
    selectedImageBlob.value = null;
    selectedAssetKind.value = '';
    if (!fileInputRef.value || fileInputRef.value.value === null) return;
    fileInputRef.value.value = '';
  };

  const uploadImage = imageFileSend;
  const imageCrear = imageClear;

  return {
    fileInputRef,
    selectedImageBlob,
    selectedFileName,
    selectedFileSize,
    selectedMimeType,
    selectedAssetKind,
    selectedFileTypeLabel,
    acceptedFileTypes,
    isUploadReady,
    isImageSendNow,
    formatFileSize,
    onImageSelect,
    imageFileSend,
    uploadImage,
    imageClear,
    imageCrear,
  };
}
