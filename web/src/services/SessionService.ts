import request from '~/request'
import {ResultData} from "~/request/type";
import {ElMessage} from "element-plus";

const handleResponseCode = (res: ResultData) => {
    let msg = null
    switch (res.code) {
        case 10001:
            msg = res.msg ? res.msg : "Invalid key file!"
            break
        case 10002:
            msg = res.msg ? res.msg : "Connect error!"
            break
    }
    if (msg) {
        ElMessage({
            showClose: true,
            message: msg,
            type: "warning",
            duration: 3000,
        })
    }
}

export function newSession() {

}

export function testSession(data: any): Promise<any> {
    return new Promise<any>((resolve, reject) => {
        request.post("http://127.0.0.1:8002/session/test", data).then(res => {
            let resData: ResultData = res.data
            if (resData.code === 0) {
                resolve(resData)
            } else {
                handleResponseCode(resData)
                reject(resData.msg)
            }
        }).catch(e => {
            reject(e.message)
        })
    })
}