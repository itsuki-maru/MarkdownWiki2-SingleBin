import { ref } from 'vue';
import type { Ref } from 'vue';
import { AxiosError } from 'axios';
import { imageUploadUrl } from '@/router/urls';
import { baseUrl } from '@/setting';
import { useImageStore } from '@/stores/images';
import type { ImageData } from '@/interface';
import apiClient from '@/axiosClient';
import { isPDF, isMP4 } from '@/utils/markedSetup';

export function useImageUpload(
  showProgressModal: Ref<boolean>,
  onMessage: (msg: string) => void,
  onUploadSuccess: (markdownStr: string) => void,
) {
  const selectedImageBlob = ref<Blob | null>(null);
  const selectedFileName = ref<string>('');
  const isImageSendNow = ref(false);

  const onImageSelect = async (): Promise<void> => {
    const element = document.getElementById('image1')! as HTMLInputElement;
    if (element.value === '' || element.value === null) {
      onMessage('画像ファイルを選択してください。');
      return;
    }

    const file = element.files!;
    const fileObj = file[0]!;
    const fileName = fileObj.name;

    const arrowMimeTypes = [
      'image/jpeg',
      'image/png',
      'image/webp',
      'image/gif',
      'video/mp4',
      'application/pdf',
    ];
    if (!arrowMimeTypes.includes(fileObj.type)) {
      onMessage('許可されていない形式のファイルです。');
      imageCrear();
      return;
    }

    if (fileObj.type.startsWith('image/')) {
      try {
        showProgressModal.value = true;
        selectedImageBlob.value = await resizeImageWithCanvas(fileObj);
      } catch (error) {
        console.error('リサイズエラー: ', error);
        selectedImageBlob.value = null;
      } finally {
        showProgressModal.value = false;
      }
    } else {
      selectedImageBlob.value = fileObj;
    }
    selectedFileName.value = fileName;
  };

  const resizeImageWithCanvas = (file: File): Promise<Blob> => {
    return new Promise((resolve, reject) => {
      const img = new Image();
      img.onload = () => {
        const maxWidth = 1280;
        const maxHeight = 720;

        const { width, height } = caluculateDimensions(img.width, img.height, maxWidth, maxHeight);

        const canvas = document.createElement('canvas');
        canvas.width = width;
        canvas.height = height;

        const ctx = canvas.getContext('2d');
        if (!ctx) {
          reject(new Error('Canvas contextの取得に失敗しました。'));
          return;
        }
        ctx.drawImage(img, 0, 0, width, height);

        canvas.toBlob(
          (blob) => {
            if (blob) {
              resolve(blob);
            } else {
              reject(new Error('Blobの生成に失敗しました。'));
            }
          },
          file.type,
          0.8,
        );
      };
      img.onerror = () => reject(new Error('画像の読み込みに失敗しました。'));
      img.src = URL.createObjectURL(file);
    });
  };

  const caluculateDimensions = (
    width: number,
    height: number,
    maxWidth: number,
    maxHeight: number,
  ) => {
    if (height < width) {
      if (width > maxWidth || height > maxHeight) {
        const widthRatio = maxWidth / width;
        const heightRatio = maxHeight / height;
        const ratio = Math.min(widthRatio, heightRatio);
        return {
          width: Math.floor(width * ratio),
          height: Math.floor(height * ratio),
        };
      }
      return { width, height };
    } else {
      if (height > maxWidth || width > maxHeight) {
        const widthRatio = maxWidth / height;
        const heightRatio = maxHeight / width;
        const ratio = Math.min(widthRatio, heightRatio);
        return {
          width: Math.floor(width * ratio),
          height: Math.floor(height * ratio),
        };
      }
      return { width, height };
    }
  };

  const uploadImage = async (): Promise<void> => {
    if (isImageSendNow.value === true) {
      return;
    } else {
      isImageSendNow.value = true;
    }
    showProgressModal.value = true;

    if (!selectedImageBlob.value) {
      onMessage('ファイルを選択してください。');
      isImageSendNow.value = false;
      showProgressModal.value = false;
      return;
    }

    const payload = new FormData();
    payload.append('upload_file', selectedImageBlob.value, selectedFileName.value);

    try {
      const response = await apiClient.post(imageUploadUrl, payload);

      const newImageData: ImageData = {
        id: response.data['new_image_id'],
        user_id: response.data['user_id'],
        filename: response.data['filename'],
        uuid_filename: response.data['uuid_filename'],
      };
      const imageStore = useImageStore();
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
      onUploadSuccess(imageUrlMarkdown);
      imageCrear();
    } catch (error) {
      if (apiClient.isAxiosError(error)) {
        const axiosError = error as AxiosError;
        if (axiosError.response) {
          const status = axiosError.response.status;
          switch (status) {
            case 400:
              onMessage(`${axiosError.response.data}`);
              console.error('Please remove spaces from the file name.', axiosError.response.data);
              break;
            case 401:
              console.error('No token provided.', axiosError.response.data);
              break;
            case 500:
              onMessage(`${axiosError.response.data}`);
              console.error('Server error, please try again later', axiosError.response.data);
              break;
            default:
              onMessage(`${axiosError.response.data}`);
              console.error(`An error occurred: ${status}`, axiosError.response.data);
              break;
          }
        }
      }
    } finally {
      selectedImageBlob.value = null;
      selectedFileName.value = '';
      isImageSendNow.value = false;
      showProgressModal.value = false;
    }
  };

  const imageCrear = (): void => {
    selectedFileName.value = '';
    selectedImageBlob.value = null;
    let imageContent = document.getElementById('image1')! as HTMLInputElement;
    if (imageContent.value === null) {
      return;
    } else {
      imageContent.value = '';
    }
  };

  return {
    selectedImageBlob,
    selectedFileName,
    isImageSendNow,
    onImageSelect,
    uploadImage,
    imageCrear,
  };
}
