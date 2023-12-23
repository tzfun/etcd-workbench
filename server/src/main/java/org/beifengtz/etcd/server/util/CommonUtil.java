package org.beifengtz.etcd.server.util;

import com.google.common.base.Joiner;
import com.google.common.base.Splitter;
import io.etcd.jetcd.ByteSequence;
import io.netty.channel.Channel;
import io.netty.handler.codec.http.FullHttpRequest;
import org.beifengtz.jvmm.common.util.SignatureUtil;
import org.beifengtz.jvmm.convey.channel.ChannelUtil;

import javax.crypto.BadPaddingException;
import javax.crypto.Cipher;
import javax.crypto.IllegalBlockSizeException;
import javax.crypto.NoSuchPaddingException;
import java.nio.charset.StandardCharsets;
import java.security.InvalidKeyException;
import java.security.KeyFactory;
import java.security.NoSuchAlgorithmException;
import java.security.interfaces.RSAPrivateKey;
import java.security.interfaces.RSAPublicKey;
import java.security.spec.InvalidKeySpecException;
import java.security.spec.PKCS8EncodedKeySpec;
import java.security.spec.X509EncodedKeySpec;
import java.util.ArrayList;
import java.util.Base64;
import java.util.List;

import static java.nio.charset.StandardCharsets.UTF_8;

/**
 * description: TODO
 * date: 10:19 2023/5/26
 *
 * @author beifengtz
 */
public class CommonUtil {

    public static final String EMPTY_STR = "";
    public static final String EMPTY_IP = "::::";

    public static ByteSequence toByteSequence(String str) {
        if (str == null || str.isEmpty()) {
            return ByteSequence.EMPTY;
        }
        return ByteSequence.from(str, UTF_8);
    }

    public static String toString(ByteSequence bs) {
        byte[] bytes = bs.getBytes();
        if (bytes.length == 1 && bytes[0] == 0) {
            return EMPTY_STR;
        }
        return bs.toString(UTF_8);
    }

    /**
     * 分段加密
     *
     * @param content   明文
     * @param publicKey 公钥
     * @param splitter  分割字符
     * @return 密文
     * @throws Exception 加密异常
     */
    public static String RSAEncryptPartly(String content, String publicKey, String splitter) throws Exception {
        final int maxLen = 117; //  最大字节数
        byte[] bytes = content.getBytes(StandardCharsets.UTF_8);
        final int strLen = bytes.length;

        if (strLen <= maxLen) {
            return SignatureUtil.RSAEncrypt(content, publicKey);
        } else {
            byte[] decodedPublicKey = Base64.getDecoder().decode(publicKey);
            int idx = 0;
            List<String> arr = new ArrayList<>(strLen / maxLen + 1);
            while (idx < strLen) {
                int size = Math.min(strLen, idx + maxLen) - idx;
                byte[] temp = new byte[size];
                System.arraycopy(bytes, idx, temp, 0, size);

                arr.add(RSAEncrypt(temp, decodedPublicKey));
                idx += Math.min(maxLen, strLen - idx);
            }
            return Joiner.on(splitter).join(arr);
        }
    }

    private static String RSAEncrypt(byte[] content, byte[] publicKey) throws Exception {
        RSAPublicKey pubKey = (RSAPublicKey) KeyFactory.getInstance("RSA").generatePublic(new X509EncodedKeySpec(publicKey));
        Cipher cipher = Cipher.getInstance("RSA");
        cipher.init(Cipher.ENCRYPT_MODE, pubKey);
        return Base64.getEncoder().encodeToString(cipher.doFinal(content));
    }

    /**
     * 分段式解密
     *
     * @param content    密文
     * @param privateKey 私钥
     * @param splitter   分割字符
     * @return 明文
     * @throws Exception 解密异常
     */
    public static String RSADecryptPartly(String content, String privateKey, String splitter) throws Exception {
        Iterable<String> parts = Splitter.on(splitter).split(content);
        byte[] decodedPrivateKey = Base64.getDecoder().decode(privateKey);
        byte[] bytes = null;
        for (String part : parts) {
            byte[] temp1 = RSADecrypt(part, decodedPrivateKey);
            if (bytes == null) {
                bytes = temp1;
            } else {
                byte[] temp2 = new byte[bytes.length + temp1.length];
                System.arraycopy(bytes, 0, temp2, 0, bytes.length);
                System.arraycopy(temp1, 0, temp2, bytes.length, temp1.length);
                bytes = temp2;
            }
        }
        assert bytes != null;
        return new String(bytes);
    }

    private static byte[] RSADecrypt(String str, byte[] privateKey) throws NoSuchAlgorithmException, InvalidKeySpecException,
            NoSuchPaddingException, InvalidKeyException, IllegalBlockSizeException, BadPaddingException {
        byte[] inputByte = Base64.getDecoder().decode(str.getBytes(StandardCharsets.UTF_8));
        RSAPrivateKey priKey = (RSAPrivateKey) KeyFactory.getInstance("RSA").generatePrivate(new PKCS8EncodedKeySpec(privateKey));
        Cipher cipher = Cipher.getInstance("RSA");
        cipher.init(Cipher.DECRYPT_MODE, priKey);
        return cipher.doFinal(inputByte);
    }

    public static String getHttpRealIp(FullHttpRequest msg) {
        String ip = msg.headers().get("X-Real-IP");
        if (ip == null) {
            ip = EMPTY_IP;
        }
        return ip;
    }
}
