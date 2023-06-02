import Axios, {AxiosError, AxiosInstance, AxiosRequestConfig, AxiosResponse, InternalAxiosRequestConfig,} from "axios";
import {Request, ResultData} from "~/request/type";

import {ElMessage} from "element-plus";
import {_loading} from "~/util/CommonUtil";

/**
 * 封装的 element-plus 的消息提示框
 * @param msg
 */
const message = (msg: string) => {
    ElMessage({
        message: msg,
        type: "warning",
        duration: 1500,
    });
};

/**
 * 默认 create Axios 的配置参数
 */
const defaultConfig: AxiosRequestConfig = {
    baseURL: "",
    timeout: 10000, //10秒超时
    withCredentials: true,
    responseType: "json",
    validateStatus() {
        return true;
    },
    transformRequest: [
        (data) => {
            data = JSON.stringify(data);
            return data;
        },
    ],
    transformResponse: [
        (data) => {
            if (typeof data === "string" && (data.startsWith("{") || data.startsWith("["))) {
                data = JSON.parse(data);
            }
            return data;
        },
    ],
    headers: {
        Accept: "application/json, text/plain, */*",
        "Content-Type": "application/json",
        "X-Requested-With": "XMLHttpRequest",
    },
};

/**
 * Axios create的时候后去的配置参数
 * @param config
 */
const getConfig = (config?: AxiosRequestConfig): AxiosRequestConfig | undefined => {
    if (!config) return config;
    return defaultConfig;
};

const handleResponseCode = (res: ResultData, notifyError: boolean | undefined = true) => {
    let msg = null
    switch (res.code) {
        case 10001:
            msg = res.msg ? res.msg : "Invalid key file!"
            break
        case 10002:
            msg = res.msg ? res.msg : "Connect error!"
            break
    }
    if (msg && notifyError) {
        ElMessage({
            showClose: true,
            message: msg,
            type: "warning",
            duration: 3000,
        })
    }
}

const handleAxiosError = (e: AxiosError, notifyError: boolean | undefined = true) => {
    if (notifyError) {
        ElMessage({
            showClose: true,
            message: e.message,
            type: "error",
            duration: 3000,
        })
    }
}

/**
 * 自定义封装的Axios 类
 */
class EnclosureHttp {
    constructor() {
        this.httpInterceptorsRequest();
        this.httpInterceptorsResponse();
    }

    /**
     * Axios 实例
     * @private
     */
    private static axiosInstance: AxiosInstance = Axios.create(getConfig());

    /**
     * 请求拦截
     * @private
     */
    private httpInterceptorsRequest(): void {
        EnclosureHttp.axiosInstance.interceptors.request.use(
            (config: InternalAxiosRequestConfig<any>) => {
                /*
                 * 在请求发出去之前作出一下处理
                 * */
                // console.log("config=>:", config);
                return config;
            },
            (err) => {
                return Promise.reject(err);
            }
        );
    }

    /**
     * 响应拦截
     * @private
     */
    private httpInterceptorsResponse(): void {
        EnclosureHttp.axiosInstance.interceptors.response.use(
            (response: AxiosResponse) => {
                /*
                 *   对响应的数据作出一些处理
                 * */
                const {status} = response;
                let msg = "";
                if (status < 200 || status >= 300) {
                    // 处理http错误，抛到业务代码
                    if (typeof response.data === "string") {
                        msg = "打盹了！！！";
                        response.data = {msg};
                    } else {
                        response.data.msg = msg;
                    }
                }
                return response;
            },
            (error: AxiosError) => {
                //请求出错的验证
                const {response} = error;
                if (response) {
                    this.errorHandle(response.status, response.statusText);
                    return Promise.reject(response);
                } else {
                    // 处理断网的情况
                    // eg:请求超时或断网时，更新state的network状态
                    // network状态在app.vue中控制着一个全局的断网提示组件的显示隐藏
                    // 后续增加断网情况下做的一些操作
                    return Promise.reject(error);
                }
            }
        );
    }

    /**
     * 请求失败后的错误统一处理
     * @param status 请求失败的状态码
     * @param other
     */
    private errorHandle = (status: number, other: string) => {
        // 状态码判断
        switch (status) {
            case -1: // -1: 未登录状态，跳转登录页
                message("未登录状态");
                break;
            case 403: // 403 token过期
                message("登录过期，请重新登录");
                break;
            case 404: // 404请求不存在
                message("请求错误！！！");
                break;
            default:
                message(other);
        }
    };

    /**
     * Axios init GET方法
     * @param url 路径
     * @param params 参数
     * @param config
     * @param notifyError
     */
    public get: Request = (
        url: string,
        params?: unknown,
        config?: AxiosRequestConfig,
        notifyError?: boolean
    ) => {
        return new Promise<any>((resolve, reject) => {
            let loading = _loading()
            EnclosureHttp.axiosInstance.get(url, {params, ...config}).then(res => {
                let resData: ResultData = res.data
                if (resData.code === 0) {
                    resolve(resData.data)
                } else {
                    handleResponseCode(resData, notifyError)
                    reject(resData.msg)
                }
            }).catch((e: AxiosError) => {
                handleAxiosError(e, notifyError)
                reject(e.message)
            }).finally(() => {
                loading.close()
            })
        });
    };

    /**
     * Axios init POST 方法
     * @param url 路径
     * @param data 参数
     * @param config
     * @param notifyError
     */
    public post: Request = (
        url: string,
        data: unknown = {},
        config?: AxiosRequestConfig,
        notifyError?: boolean
    ) => {
        return new Promise<any>((resolve, reject) => {
            let loading = _loading()
            EnclosureHttp.axiosInstance.post(url, data, config).then(res => {
                let resData: ResultData = res.data
                if (resData.code === 0) {
                    resolve(resData.data)
                } else {
                    handleResponseCode(resData, notifyError)
                    reject(resData.msg)
                }
            }).catch((e: AxiosError) => {
                handleAxiosError(e, notifyError)
                reject(e.message)
            }).finally(() => {
                loading.close()
            })
        });
    };
}

export default EnclosureHttp;