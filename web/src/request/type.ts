import {AxiosRequestConfig} from "axios";

export interface ResultData {
    code: number,
    msg?: string;
    data?: any;
}

/**
 *
 */
export interface Request {
    <T>(
        url: string,
        params?: Record<string, unknown> | string | null,
        config?: AxiosRequestConfig,
        notifyError?: boolean,
        withLoading?: boolean
    ): Promise<ResultData>;
}
