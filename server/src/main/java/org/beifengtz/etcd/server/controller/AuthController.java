package org.beifengtz.etcd.server.controller;

import com.google.common.cache.Cache;
import com.google.common.cache.CacheBuilder;
import lombok.extern.slf4j.Slf4j;
import org.beifengtz.etcd.server.config.Configuration;
import org.beifengtz.etcd.server.config.Mapping;
import org.beifengtz.etcd.server.config.ResultCode;
import org.beifengtz.etcd.server.entity.TokenPayload;
import org.beifengtz.etcd.server.entity.vo.CheckLoginVO;
import org.beifengtz.etcd.server.entity.vo.ResultVO;
import org.beifengtz.etcd.server.util.CommonUtil;
import org.beifengtz.jvmm.common.util.FileUtil;
import org.beifengtz.jvmm.common.util.SignatureUtil;
import org.beifengtz.jvmm.common.util.StringUtil;
import org.beifengtz.jvmm.convey.annotation.HttpController;
import org.beifengtz.jvmm.convey.annotation.HttpRequest;
import org.beifengtz.jvmm.convey.annotation.RequestBody;
import org.beifengtz.jvmm.convey.annotation.RequestParam;
import org.beifengtz.jvmm.convey.enums.Method;

import java.io.File;
import java.nio.charset.StandardCharsets;
import java.util.Base64;
import java.util.Objects;
import java.util.concurrent.TimeUnit;

/**
 * description: TODO
 * date: 16:27 2023/12/8
 *
 * @author beifengtz
 */
@HttpController
@Slf4j
public class AuthController {

    private static final long TOKEN_EXPIRE_MILLIS = TimeUnit.DAYS.toMillis(7);
    private static final Cache<String, TokenPayload> TOKEN_CACHE = CacheBuilder.newBuilder().build();

    static {
        Configuration config = Configuration.INSTANCE;
        try {
            for (String user : config.getUsers().keySet()) {
                File tokenFile = config.getUserTokenFile(user);
                if (tokenFile.exists()) {
                    String token = FileUtil.readFileToString(tokenFile, StandardCharsets.UTF_8);
                    byte[] bytes = SignatureUtil.AESDecrypt(Base64.getDecoder().decode(token), Configuration.INSTANCE.getConfigEncryptKey());
                    TokenPayload payload = TokenPayload.parseFrom(new String(bytes, StandardCharsets.UTF_8));
                    if (Objects.equals(Configuration.INSTANCE.getUsers().get(user), payload.getPassword())
                            && System.currentTimeMillis() - payload.getCreateTime() < TOKEN_EXPIRE_MILLIS) {
                        TOKEN_CACHE.put(token, payload);
                        log.debug("Loaded user `{}` token form file", user);
                    } else {
                        log.debug("User `{}` token invalid and delete file", user);
                        FileUtil.delFile(tokenFile);
                    }
                }
            }
            log.info("Loaded user token from file");
        } catch (Exception e) {
            log.error("Load token failed" + e.getMessage(), e);
        }
    }

    /**
     * 验证 token 并获得 user
     *
     * @param token token
     * @return user
     */
    public static String verifyToken(String token) {
        if (StringUtil.isEmpty(token)) {
            throw new IllegalStateException();
        }
        TokenPayload payload = TOKEN_CACHE.getIfPresent(token);
        if (payload == null) {
            throw new IllegalStateException();
        }
        if (System.currentTimeMillis() - payload.getCreateTime() >= TOKEN_EXPIRE_MILLIS) {
            TOKEN_CACHE.invalidate(token);
            throw new IllegalStateException();
        }
        return payload.getUser();
    }

    @HttpRequest(value = Mapping.PUBLIC_API_PREFIX + "/auth/login", method = Method.GET)
    public ResultVO login(@RequestParam String user, @RequestParam String code) throws Exception {
        if (Configuration.INSTANCE.isEnableAuth()) {
            String password = Configuration.INSTANCE.getUsers().get(user);
            if (password == null) {
                return ResultCode.LOGIN_FAILED.result("Incorrect username or password", null);
            }
            if (Objects.equals(SignatureUtil.MD5(user + "," + password), code)) {
                TokenPayload payload = new TokenPayload();
                payload.setUser(user);
                payload.setPassword(password);
                payload.setCreateTime(System.currentTimeMillis());
                String token = Base64.getEncoder().encodeToString(SignatureUtil.AESEncrypt(payload.toString().getBytes(StandardCharsets.UTF_8),
                        Configuration.INSTANCE.getConfigEncryptKey()));
                File file = Configuration.INSTANCE.getUserTokenFile(user);
                FileUtil.writeByteArrayToFile(file, token.getBytes(StandardCharsets.UTF_8));
                TOKEN_CACHE.put(token, payload);
                return ResultCode.OK.result(token);
            } else {
                return ResultCode.LOGIN_FAILED.result("Incorrect username or password", null);
            }
        } else {
            return ResultCode.NOT_SUPPORTED.result();
        }
    }

    @HttpRequest(value = Mapping.PUBLIC_API_PREFIX + "/auth/check_login", method = Method.POST)
    public ResultVO checkLogin(@RequestBody String token) {
        CheckLoginVO result = new CheckLoginVO();
        Configuration conf = Configuration.INSTANCE;
        result.setEnableAuth(conf.isEnableAuth());
        if (result.isEnableAuth()) {
            try {
                verifyToken(token);
            } catch (Exception e) {
                result.setNeedLogin(true);
            }
        }
        String[] versionInfo = CommonUtil.getVersionInfo();
        result.setVersion(versionInfo[0]);
        result.setBuildHash(versionInfo[1]);
        result.setEnableHeartbeat(conf.isEnableHeartbeat());
        return ResultCode.OK.result(result);
    }
}
