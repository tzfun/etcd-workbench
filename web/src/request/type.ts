import { AxiosRequestConfig } from "axios";

export interface CustomSuccessData<T> {
    code: number;
    msg?: string;
    data: T;
    [keys: string]: unknown;
}

/**
 *
 */
export interface Request {
    <T>(
        url: string,
        params?: Record<string, unknown>,
        config?: AxiosRequestConfig
    ): Promise<CustomSuccessData<T>>;
}