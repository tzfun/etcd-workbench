import request from '~/request'

export function newSession() {

}

export function testSession(data: any): Promise<any> {
    return new Promise<any>((resolve, reject) => {
        request.post("http://127.0.0.1:8001/session/test", data).then(data => {
            if (data.code === 0) {
                resolve(data.data)
            } else {
                reject(data.msg)
            }
        }).catch(e => {
            reject(e.message)
        })
    })
}