package org.beifengtz.etcd.server.controller;

import com.google.common.cache.Cache;
import com.google.common.cache.CacheBuilder;
import com.google.gson.JsonParser;
import lombok.extern.slf4j.Slf4j;
import org.beifengtz.etcd.server.config.Configuration;
import org.beifengtz.etcd.server.config.Mapping;
import org.beifengtz.etcd.server.config.ResultCode;
import org.beifengtz.etcd.server.entity.dto.CodedDTO;
import org.beifengtz.etcd.server.entity.vo.ResultVO;
import org.beifengtz.etcd.server.util.CommonUtil;
import org.beifengtz.jvmm.common.util.FileUtil;
import org.beifengtz.jvmm.common.util.SignatureUtil;
import org.beifengtz.jvmm.common.util.StringUtil;
import org.beifengtz.jvmm.common.util.meta.PairKey;
import org.beifengtz.jvmm.convey.annotation.HttpController;
import org.beifengtz.jvmm.convey.annotation.HttpRequest;
import org.beifengtz.jvmm.convey.annotation.RequestAttr;
import org.beifengtz.jvmm.convey.annotation.RequestBody;
import org.beifengtz.jvmm.convey.annotation.RequestParam;
import org.beifengtz.jvmm.convey.enums.Method;

import java.io.File;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.security.spec.InvalidKeySpecException;
import java.util.Base64;
import java.util.HashMap;
import java.util.Map;
import java.util.Objects;
import java.util.concurrent.TimeUnit;

/**
 * description: TODO
 * date: 15:01 2023/12/7
 *
 * @author beifengtz
 */
@HttpController
@Slf4j
public class ManageController {
    private static final Cache<String, String> KEY_MAP = CacheBuilder.newBuilder().expireAfterWrite(1, TimeUnit.MINUTES).build();

    @SuppressWarnings("unchecked")
    public static <T> T decode(CodedDTO dto, Class<T> clazz) throws Exception {
        String key = KEY_MAP.getIfPresent(dto.getCode());
        if (key == null) {
            throw new InvalidKeySpecException();
        }
        String str = CommonUtil.RSADecryptPartly(dto.getData(), key, "|");
        if (clazz.isAssignableFrom(String.class)) {
            return (T) str;
        }
        return StringUtil.getGson().fromJson(str, clazz);
    }

    @HttpRequest(value = Mapping.PRIVATE_API_PREFIX + "/auth/ping", method = Method.GET)
    public ResultVO ping(@RequestParam String code) throws Exception {
        checkCode(code);
        PairKey<String, String> pairKey = SignatureUtil.genRSAKeyPair();
        KEY_MAP.put(code, pairKey.getRight());
        return ResultCode.PARAM_FORMAT_ERROR.result(pairKey.getLeft());
    }

    @HttpRequest(value = Mapping.PRIVATE_API_PREFIX + "/config/save", method = Method.POST)
    public ResultVO saveConfig(@RequestBody CodedDTO codedData, @RequestAttr String user) throws Exception {
        String content = decode(codedData, String.class);
        String dataDir = Configuration.INSTANCE.getDataDir();
        String configName = SignatureUtil.MD5(JsonParser.parseString(content).getAsJsonObject().get("name").toString());
        File file = new File(dataDir + "/" + user + "/" + configName);
        FileUtil.writeByteArrayToFile(file, SignatureUtil.AESEncrypt(content.getBytes(StandardCharsets.UTF_8), Configuration.INSTANCE.getConfigEncryptKey()));
        log.debug("Saved config file {}", file);
        return ResultCode.OK.result(configName);
    }

    @HttpRequest(value = Mapping.PRIVATE_API_PREFIX + "/config/delete", method = Method.GET)
    public ResultVO deleteConfig(@RequestParam String key, @RequestAttr String user) {
        String dataDir = Configuration.INSTANCE.getDataDir();
        File file = new File(dataDir + "/" + user + "/" + key);
        log.debug("Deleted config file {}", file);
        FileUtil.delFile(file);
        return ResultCode.OK.result();
    }

    @HttpRequest(value = Mapping.PRIVATE_API_PREFIX + "/config/list", method = Method.GET)
    public ResultVO listConfig(@RequestAttr String user) throws Exception {
        String dataDir = Configuration.INSTANCE.getDataDir();
        File userDir = new File(dataDir, user);
        File[] files = userDir.listFiles();
        Map<String, String> result = new HashMap<>();
        if (files != null) {
            for (File file : files) {
                byte[] bytes = Files.readAllBytes(file.toPath());
                try {
                    String content = Base64.getEncoder().encodeToString(SignatureUtil.AESDecrypt(bytes, Configuration.INSTANCE.getConfigEncryptKey()));
                    result.put(file.getName(), content);
                } catch (Exception e) {
                    log.warn("Read config file from {} error: {}: {}", file, e.getClass(), e.getMessage());
                    FileUtil.delFile(file);
                }
            }
        }
        return ResultCode.OK.result(result);
    }

    private void checkCode(String code) {
        if (code == null || code.length() <= 2) {
            throw new IllegalArgumentException();
        }
        String parsedCode;
        try {
            int radix = Integer.parseInt(code.substring(code.length() - 2));
            parsedCode = Long.toString(Long.parseLong(code.substring(2), radix), 36);
        } catch (NumberFormatException e) {
            throw new IllegalArgumentException();
        }

        if (!Objects.equals("beifengtz", parsedCode)) {
            throw new IllegalArgumentException();
        }
    }
}
