//  mdi字体体积优化：https://www.shedloadofcode.com/blog/reduce-material-design-icons-font-to-7kb-and-automate-with-pyautogui
//  图片尺寸调整：https://www.iloveimg.com/zh-cn/resize-image
//  png转ico：https://cloudconvert.com/png-converter
import path from 'path'
import fs from 'fs'

function deleteFile(fileOrDirName) {
    let stat = fs.statSync(fileOrDirName)
    if (stat.isDirectory()) {
        for (let f of fs.readdirSync(fileOrDirName)) {
            let filePath = path.join(fileOrDirName, f)
            deleteFile(filePath)
        }
        fs.rmdirSync(fileOrDirName)
    } else {
        fs.unlinkSync(fileOrDirName)
        console.log(`deleted ==> ${fileOrDirName}`)
    }
}

function deleteFileWithPattern(dir, regexp) {
    let replaceMap = {}
    for (let f of fs.readdirSync(dir)) {
        if (f.match(regexp)) {
            fs.unlinkSync(path.join(dir, f))
            console.log(`deleted ==> ${dir}/${f}`)
            if (f.endsWith("css") || f.endsWith("js")) {
                replaceMap[f] = f + ".gz"
            }
        }
    }
    return replaceMap
}

function replaceFile(file, replaceMap) {
    let data = fs.readFileSync(file, {encoding: 'utf-8'})
    for (let key in replaceMap) {
        data = data.replaceAll(key, replaceMap[key])
        console.log(`replaced ${key} to ${replaceMap[key]}`)
    }
    fs.writeFileSync(file, data, {encoding: 'utf-8'})
}


const replaceMap = deleteFileWithPattern('./dist/assets', /^.*\.(ttf|eot|woff)$/)
// replaceFile('./dist/index.html', replaceMap)