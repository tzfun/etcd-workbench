import request from '~/request'
import {ResultData} from "~/request/type";

export function newSession() {

}

export function testSession(data: any): Promise<any> {
    return new Promise<any>((resolve, reject) => {
        request.post("http://127.0.0.1:8002/session/test", data).then(res=> {
            let resData: ResultData = res.data
            if (resData.code === 0) {
                resolve(resData)
            } else {
                reject(resData.msg)
            }
        }).catch(e => {
            reject(e.message)
        })
    })
}