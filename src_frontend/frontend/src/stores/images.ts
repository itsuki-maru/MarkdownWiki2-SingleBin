import { defineStore } from "pinia";
import type { ImageData } from "@/interface";
import { imageListGetUrl } from "@/router/urls";
import apiClient from "@/axiosClient";

interface State {
    imageList: Map<string, ImageData>;
}

export const useImageStore = defineStore({
    id: "images",
    state: (): State => {
        return {
            imageList: new Map<string, ImageData>()
        };
    },
    getters: {
        getById: (state) => {
            return (id: string): ImageData => {
                const image = state.imageList.get(id) as ImageData;
                return image;
            };
        }
    },
    actions: {
        async initList(): Promise<void> {
            try {
                const response = await apiClient.get(imageListGetUrl);
                this.imageList.clear();
                const imagesData = response.data;
                for (let key in imagesData) {
                    this.imageList.set(
                        imagesData[key]["id"], {
                            id: imagesData[key]["id"],
                            user_id: imagesData[key]["user_id"],
                            filename: imagesData[key]["filename"],
                            uuid_filename: imagesData[key]["uuid_filename"]
                        }
                    );
                }
                let sortedDsc = new Map([...this.imageList.entries()].sort((a, b) => a[0] > b[0] ? 1 : -1).reverse());
                this.imageList = sortedDsc;
            } catch (error) {
                console.log("Init List Error.");
            }
        },
        addImage(image: ImageData): void {
            this.imageList.set(image.id, {
                id: image.id,
                user_id: image.user_id,
                filename: image.filename,
                uuid_filename: image.uuid_filename
            });
            let sortedDsc = new Map([...this.imageList.entries()].sort((a, b) => a[0] > b[0] ? 1 : -1).reverse());
            this.imageList = sortedDsc;
        },
        deleteImage(image_id: string): void {
            this.imageList.delete(image_id);
        },
        queryImage(query: string): void {
            if (query === "") {
                this.initList();
                return;
            }
            let result: { id: string, user_id: string, filename: string, uuid_filename: string }[] = [];
            this.imageList.forEach((value, key) => {
                if (value.filename.includes(query)) {
                    result.push(value);
                }
            });
    
            // Sort by id in descending order
            result.sort((a, b) => b.id.localeCompare(a.id));
    
            let resultMap: Map<string, { id: string, user_id: string, filename: string, uuid_filename: string }> = new Map();
            for (let item of result) {
                resultMap.set(item.id, item);
            }
            this.imageList = resultMap;
        },
    }
})