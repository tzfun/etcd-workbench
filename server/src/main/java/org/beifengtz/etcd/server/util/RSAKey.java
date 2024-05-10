package org.beifengtz.etcd.server.util;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.math.BigInteger;
import java.security.KeyFactory;
import java.security.NoSuchAlgorithmException;
import java.security.interfaces.RSAPrivateKey;
import java.security.interfaces.RSAPublicKey;
import java.security.spec.InvalidKeySpecException;
import java.security.spec.RSAPrivateCrtKeySpec;
import java.security.spec.RSAPublicKeySpec;
import java.util.Base64;
import java.util.UnknownFormatConversionException;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

/**
 * description: RSA pem工具类
 * date: 11:05 2023/5/26
 *
 * @author beifengtz
 */
public class RSAKey {

    private static final Pattern PEM_CODE_PATTERN = Pattern.compile("--+.+?--+|[\\s\\r\\n]+");
    private static final byte[] _SeqOID = new byte[]{0x30, 0x0D, 0x06, 0x09, 0x2A, (byte) 0x86, 0x48, (byte) 0x86, (byte) 0xF7, 0x0D, 0x01, 0x01, 0x01, 0x05, 0x00};
    private static final byte[] _Ver = new byte[]{0x02, 0x01, 0x00};
    private static final Pattern XML_PATTERN = Pattern.compile("\\s*<RSAKeyValue>([<>\\/\\+=\\w\\s]+)</RSAKeyValue>\\s*");
    private static final Pattern XML_TAG_PATTERN = Pattern.compile("<(.+?)>\\s*([^<]+?)\\s*</");

    /**
     * modulus 模数，公钥、私钥都有
     **/
    private byte[] modulus;
    /**
     * publicExponent 公钥指数，公钥、私钥都有
     **/
    private byte[] publicExponent;
    /**
     * privateExponent 私钥指数，只有私钥的时候才有
     **/
    private byte[] privateExponent;

    //以下参数只有私钥才有 https://docs.microsoft.com/zh-cn/dotnet/api/system.security.cryptography.rsaparameters?redirectedfrom=MSDN&view=netframework-4.8
    private byte[] prime1;
    private byte[] prime2;
    private byte[] exponent1;
    private byte[] exponent2;
    private byte[] coefficient;

    private RSAKey() {
    }

    public RSAKey(RSAPublicKey publicKey) {
        this(publicKey, null);
    }

    public RSAKey(RSAPublicKey publicKey, RSAPrivateKey privateKey) {
        this(
                bigB(publicKey.getModulus()),
                bigB(publicKey.getPublicExponent()),
                privateKey == null ? null : bigB(privateKey.getPrivateExponent())
        );
    }

    /**
     * 通过全量的PEM字段数据构造一个密钥。
     * 公钥需传入：modulus, exponent
     * 私钥需传入：所有参数
     * 注意：所有参数首字节如果是0，必须先去掉
     *
     * @param modulus  模数
     * @param exponent 公钥指数
     * @param d        私钥指数
     * @param p        prime1
     * @param q        prime2
     * @param dp       exponent1
     * @param dq       exponent2
     * @param inverseQ coefficient
     */
    public RSAKey(byte[] modulus, byte[] exponent, byte[] d, byte[] p, byte[] q, byte[] dp, byte[] dq, byte[] inverseQ) {
        this.modulus = modulus;
        this.publicExponent = exponent;

        if (d != null) {
            this.privateExponent = bigL(d, modulus.length);

            int keyLen = modulus.length / 2;
            this.prime1 = bigL(p, keyLen);
            this.prime2 = bigL(q, keyLen);
            this.exponent1 = bigL(dp, keyLen);
            this.exponent2 = bigL(dq, keyLen);
            this.coefficient = bigL(inverseQ, keyLen);
        }
    }

    /**
     * 构造公钥
     *
     * @param modulus  模数
     * @param exponent 公钥指数
     */
    public RSAKey(byte[] modulus, byte[] exponent) {
        this(modulus, exponent, null);
    }

    /***
     * 通过公钥指数和私钥指数构造一个PEM，会反推计算出P、Q但和原始生成密钥的P、Q极小可能相同
     * 注意：所有参数首字节如果是0，必须先去掉
     * @param modulus 模数
     * @param exponent 公钥指数
     * @param privateExponent 私钥指数可以不提供，导出的PEM就只包含公钥
     **/
    public RSAKey(byte[] modulus, byte[] exponent, byte[] privateExponent) {
        this.modulus = modulus;
        this.publicExponent = exponent;

        if (privateExponent != null) {
            this.privateExponent = bigL(privateExponent, modulus.length);

            //反推P、Q
            BigInteger n = bigX(modulus);
            BigInteger e = bigX(exponent);
            BigInteger d = bigX(privateExponent);
            BigInteger p = findFactor(e, d, n);
            BigInteger q = n.divide(p);
            if (p.compareTo(q) > 0) {
                BigInteger t = p;
                p = q;
                q = t;
            }
            BigInteger exp1 = d.mod(p.subtract(BigInteger.ONE));
            BigInteger exp2 = d.mod(q.subtract(BigInteger.ONE));
            BigInteger coeff = q.modInverse(p);

            int keyLen = modulus.length / 2;
            prime1 = bigL(bigB(p), keyLen);
            prime2 = bigL(bigB(q), keyLen);
            exponent1 = bigL(bigB(exp1), keyLen);
            exponent2 = bigL(bigB(exp2), keyLen);
            coefficient = bigL(bigB(coeff), keyLen);
        }
    }

    /**
     * 转成正整数，如果是负数，需要加前导0转成正整数
     **/
    public static BigInteger bigX(byte[] bigb) {
        if (bigb[0] < 0) {
            byte[] c = new byte[bigb.length + 1];
            System.arraycopy(bigb, 0, c, 1, bigb.length);
            bigb = c;
        }
        return new BigInteger(bigb);
    }

    /**
     * BigInt导出byte整数首字节>0x7F的会加0前导，保证正整数，因此需要去掉0
     **/
    public static byte[] bigB(BigInteger bigx) {
        byte[] val = bigx.toByteArray();
        if (val[0] == 0) {
            byte[] c = new byte[val.length - 1];
            System.arraycopy(val, 1, c, 0, c.length);
            val = c;
        }
        return val;
    }

    /**
     * 某些密钥参数可能会少一位（32个byte只有31个，目测是密钥生成器的问题，只在c#生成的密钥中发现这种参数，java中生成的密钥没有这种现象），
     * 直接修正一下就行；这个问题与BigB有本质区别，不能动BigB
     **/
    public static byte[] bigL(byte[] bytes, int keyLen) {
        if (keyLen - bytes.length == 1) {
            byte[] c = new byte[bytes.length + 1];
            System.arraycopy(bytes, 0, c, 1, bytes.length);
            bytes = c;
        }
        return bytes;
    }

    /**
     * 由n e d 反推 P Q
     * <a href="https://stackoverflow.com/questions/43136036/how-to-get-a-rsaprivatecrtkey-from-a-rsaprivatekey">https://stackoverflow.com/questions/43136036/how-to-get-a-rsaprivatecrtkey-from-a-rsaprivatekey</a>
     **/
    private static BigInteger findFactor(BigInteger e, BigInteger d, BigInteger n) {
        BigInteger edMinus1 = e.multiply(d).subtract(BigInteger.ONE);
        int s = edMinus1.getLowestSetBit();
        BigInteger t = edMinus1.shiftRight(s);

        long now = System.currentTimeMillis();
        for (int aInt = 2; true; aInt++) {
            if (aInt % 10 == 0 && System.currentTimeMillis() - now > 3000) {
                throw new RuntimeException("Estimated RSA.P timeout");//测试最多循环2次，1024位的速度很快 8ms
            }

            BigInteger aPow = BigInteger.valueOf(aInt).modPow(t, n);
            for (int i = 1; i <= s; i++) {
                if (aPow.equals(BigInteger.ONE)) {
                    break;
                }
                if (aPow.equals(n.subtract(BigInteger.ONE))) {
                    break;
                }
                BigInteger aPowSquared = aPow.multiply(aPow).mod(n);
                if (aPowSquared.equals(BigInteger.ONE)) {
                    return aPow.subtract(BigInteger.ONE).gcd(n);
                }
                aPow = aPowSquared;
            }
        }
    }

    /**
     * 用PEM格式密钥对创建RSA，支持PKCS#1、PKCS#8格式的PEM
     *
     * @param pem pem格式密钥，支持PKCS#1、PKCS#8格式，且必须包含---PUBLIC KEY---或---PRIVATE KEY---标签
     * @return {@link RSAKey}
     * @throws InvalidKeySpecException 转换失败抛出
     */
    public static RSAKey fromPem(String pem) throws InvalidKeySpecException {
        if (pem.contains("PUBLIC KEY")) {
            return fromPem(pem, true);
        } else if (pem.contains("PRIVATE KEY")) {
            return fromPem(pem, false);
        } else {
            throw new IllegalArgumentException("Wrong pem format");
        }
    }

    /**
     * 用PEM格式密钥对创建RSA，支持PKCS#1、PKCS#8格式的PEM
     *
     * @param pem           pem格式密钥，支持PKCS#1、PKCS#8格式，可不用包含PEM标签
     * @param convertPublic true-按照公钥规则解析   false-按照私钥规则解析
     * @return {@link RSAKey}
     * @throws InvalidKeySpecException 转换失败抛出
     */
    public static RSAKey fromPem(String pem, boolean convertPublic) throws InvalidKeySpecException {
        String base64 = PEM_CODE_PATTERN.matcher(pem).replaceAll("");
        byte[] dataX = Base64.getDecoder().decode(base64);
        if (dataX == null) {
            throw new IllegalArgumentException("Invalid pem format");
        }
        short[] data = new short[dataX.length];
        for (int i = 0; i < dataX.length; i++) {
            data[i] = (short) (dataX[i] & 0xff);
        }
        if (convertPublic) {
            return fromPublicKey(data);
        } else {
            return fromPrivateKey(data);
        }
    }

    public static RSAKey fromPublicKey(String pem) throws InvalidKeySpecException {
        byte[] dataX = Base64.getDecoder().decode(pem);
        short[] data = new short[dataX.length];
        for (int i = 0; i < dataX.length; i++) {
            data[i] = (short) (dataX[i] & 0xff);
        }
        return fromPublicKey(data);
    }

    private static RSAKey fromPublicKey(short[] data) throws InvalidKeySpecException {
        RSAKey rsa = new RSAKey();
        int[] idx = new int[]{0};

        readLen(0x30, data, idx);

        //检测PKCS8
        int[] idx2 = new int[]{idx[0]};
        if (eq(_SeqOID, data, idx)) {
            //读取1长度
            readLen(0x03, data, idx);
            idx[0]++;//跳过0x00
            //读取2长度
            readLen(0x30, data, idx);
        } else {
            idx = idx2;
        }

        //Modulus
        rsa.modulus = readBlock(data, idx);

        //Exponent
        rsa.publicExponent = readBlock(data, idx);
        return rsa;
    }

    public static RSAKey fromPrivateKey(String pem) throws InvalidKeySpecException {
        byte[] dataX = Base64.getDecoder().decode(pem);
        short[] data = new short[dataX.length];
        for (int i = 0; i < dataX.length; i++) {
            data[i] = (short) (dataX[i] & 0xff);
        }
        return fromPrivateKey(data);
    }

    private static RSAKey fromPrivateKey(short[] data) throws InvalidKeySpecException {
        RSAKey rsa = new RSAKey();
        int[] idx = new int[]{0};
        readLen(0x30, data, idx);

        if (!eq(_Ver, data, idx)) {
            throw new UnknownFormatConversionException("Unknown pem version");
        }

        //检测PKCS8
        int[] idx2 = new int[]{idx[0]};
        if (eq(_SeqOID, data, idx)) {
            readLen(0x04, data, idx);
            readLen(0x30, data, idx);

            if (!eq(_Ver, data, idx)) {
                throw new UnknownFormatConversionException("Invalid pem version");
            }
        } else {
            idx = idx2;
        }

        rsa.modulus = readBlock(data, idx);
        rsa.publicExponent = readBlock(data, idx);
        int keyLen = rsa.modulus.length;
        rsa.privateExponent = bigL(readBlock(data, idx), keyLen);
        keyLen = keyLen / 2;
        rsa.prime1 = bigL(readBlock(data, idx), keyLen);
        rsa.prime2 = bigL(readBlock(data, idx), keyLen);
        rsa.exponent1 = bigL(readBlock(data, idx), keyLen);
        rsa.exponent2 = bigL(readBlock(data, idx), keyLen);
        rsa.coefficient = bigL(readBlock(data, idx), keyLen);
        return rsa;
    }

    /**
     * 读取长度
     **/
    private static int readLen(int first, short[] data, int[] idxO) throws InvalidKeySpecException {
        int idx = idxO[0];
        try {
            if (data[idx] == first) {
                idx++;
                if (data[idx] == 0x81) {
                    idx++;
                    return data[idx++];
                } else if (data[idx] == 0x82) {
                    idx++;
                    return (((int) data[idx++]) << 8) + data[idx++];
                } else if (data[idx] < 0x80) {
                    return data[idx++];
                }
            }
            throw new InvalidKeySpecException("Failed to extract data from PEM");
        } finally {
            idxO[0] = idx;
        }
    }

    /**
     * 读取块数据
     **/
    private static byte[] readBlock(short[] data, int[] idxO) throws InvalidKeySpecException {
        int idx = idxO[0];
        try {
            int len = readLen(0x02, data, idxO);
            idx = idxO[0];
            if (data[idx] == 0x00) {
                idx++;
                len--;
            }
            byte[] val = new byte[len];
            for (int i = 0; i < len; i++) {
                val[i] = (byte) data[idx + i];
            }
            idx += len;
            return val;
        } finally {
            idxO[0] = idx;
        }
    }

    /**
     * 比较data从idx位置开始是否是bytes内容
     **/
    private static boolean eq(byte[] byts, short[] data, int[] idxO) {
        int idx = idxO[0];
        try {
            for (int i = 0; i < byts.length; i++, idx++) {
                if (idx >= data.length) {
                    return false;
                }
                if ((byts[i] & 0xff) != data[idx]) {
                    return false;
                }
            }
            return true;
        } finally {
            idxO[0] = idx;
        }
    }

    /***
     * 将RSA中的密钥对转换成PEM PKCS#8格式
     * @param convertToPublic 等于true时含私钥的RSA将只返回公钥，仅含公钥的RSA不受影响
     * @return 公钥如：-----BEGIN RSA PUBLIC KEY-----，私钥如：-----BEGIN RSA PRIVATE KEY-----
     */
    public String toPemPKCS1(boolean convertToPublic) throws IOException {
        return toPem(convertToPublic, false, false);
    }

    /***
     * 将RSA中的密钥对转换成PEM PKCS#8格式
     * @param convertToPublic 等于true时含私钥的RSA将只返回公钥，仅含公钥的RSA不受影响
     * @return 公钥如：-----BEGIN PUBLIC KEY-----，私钥如：-----BEGIN PRIVATE KEY-----
     */
    public String toPemPKCS8(boolean convertToPublic) throws IOException {
        return toPem(convertToPublic, true, true);
    }

    /***
     * 将RSA中的密钥对转换成PEM格式
     * @param convertToPublic 等于true时含私钥的RSA将只返回公钥，仅含公钥的RSA不受影响
     * @param privateUsePKCS8 私钥的返回格式，等于true时返回PKCS#8格式（-----BEGIN PRIVATE KEY-----），否则返回PKCS#1格式（-----BEGIN RSA PRIVATE KEY-----），返回公钥时此参数无效；两种格式使用都比较常见
     * @param publicUsePKCS8 公钥的返回格式，等于true时返回PKCS#8格式（-----BEGIN PUBLIC KEY-----），否则返回PKCS#1格式（-----BEGIN RSA PUBLIC KEY-----），返回私钥时此参数无效；一般用的多的是true PKCS#8格式公钥，PKCS#1格式公钥似乎比较少见
     * @return 公钥如：-----BEGIN PUBLIC KEY-----，私钥如：-----BEGIN PRIVATE KEY-----
     */
    public String toPem(boolean convertToPublic, boolean privateUsePKCS8, boolean publicUsePKCS8) throws IOException {
        //https://www.jianshu.com/p/25803dd9527d
        //https://www.cnblogs.com/ylz8401/p/8443819.html
        //https://blog.csdn.net/jiayanhui2877/article/details/47187077
        //https://blog.csdn.net/xuanshao_/article/details/51679824
        //https://blog.csdn.net/xuanshao_/article/details/51672547

        ByteArrayOutputStream ms = new ByteArrayOutputStream();

        if (this.privateExponent == null || convertToPublic) {
            //  生成公钥
            //  写入总字节数，不含本段长度，额外需要24字节的头，后续计算好填入
            ms.write(0x30);
            int index1 = ms.size();

            //PKCS8 多一段数据
            int index2 = -1, index3 = -1;
            if (publicUsePKCS8) {
                //  固定内容
                // encoded OID sequence for PKCS #1 rsaEncryption szOID_RSA_RSA = "1.2.840.113549.1.1.1"
                ms.write(_SeqOID);

                //从0x00开始的后续长度
                ms.write(0x03);
                index2 = ms.size();
                ms.write(0x00);

                //后续内容长度
                ms.write(0x30);
                index3 = ms.size();
            }

            //写入Modulus
            writeBlock(modulus, ms);

            //写入Exponent
            writeBlock(publicExponent, ms);


            //计算空缺的长度
            byte[] byts = ms.toByteArray();

            if (index2 != -1) {
                byts = writeLen(index3, byts, ms);
                byts = writeLen(index2, byts, ms);
            }
            byts = writeLen(index1, byts, ms);


            String flag = " PUBLIC KEY";
            if (!publicUsePKCS8) {
                flag = " RSA" + flag;
            }
            return "-----BEGIN" + flag + "-----\n" + textBreak(Base64.getEncoder().encodeToString(byts), 64) + "\n-----END" + flag + "-----";
        } else {
            //  生成私钥
            //  写入总字节数，后续写入
            ms.write(0x30);
            int index1 = ms.size();

            //写入版本号
            ms.write(_Ver);

            //PKCS8 多一段数据
            int index2 = -1, index3 = -1;
            if (privateUsePKCS8) {
                //固定内容
                ms.write(_SeqOID);

                //后续内容长度
                ms.write(0x04);
                index2 = ms.size();

                //后续内容长度
                ms.write(0x30);
                index3 = ms.size();

                //写入版本号
                ms.write(_Ver);
            }

            //写入数据
            writeBlock(modulus, ms);
            writeBlock(publicExponent, ms);
            writeBlock(privateExponent, ms);
            writeBlock(prime1, ms);
            writeBlock(prime2, ms);
            writeBlock(exponent1, ms);
            writeBlock(exponent2, ms);
            writeBlock(coefficient, ms);


            //计算空缺的长度
            byte[] byts = ms.toByteArray();

            if (index2 != -1) {
                byts = writeLen(index3, byts, ms);
                byts = writeLen(index2, byts, ms);
            }
            byts = writeLen(index1, byts, ms);


            String flag = " PRIVATE KEY";
            if (!privateUsePKCS8) {
                flag = " RSA" + flag;
            }
            return "-----BEGIN" + flag + "-----\n" + textBreak(Base64.getEncoder().encodeToString(byts), 64) + "\n-----END" + flag + "-----";
        }
    }

    /**
     * 写入一个长度字节码
     **/
    private static void writeLenByte(int len, ByteArrayOutputStream ms) {
        if (len < 0x80) {
            ms.write((byte) len);
        } else if (len <= 0xff) {
            ms.write(0x81);
            ms.write((byte) len);
        } else {
            ms.write(0x82);
            ms.write((byte) (len >> 8 & 0xff));
            ms.write((byte) (len & 0xff));
        }
    }

    /**
     * 写入一块数据
     **/
    private static void writeBlock(byte[] byts, ByteArrayOutputStream ms) throws IOException {
        boolean addZero = ((byts[0] & 0xff) >> 4) >= 0x8;
        ms.write(0x02);
        int len = byts.length + (addZero ? 1 : 0);
        writeLenByte(len, ms);

        if (addZero) {
            ms.write(0x00);
        }
        ms.write(byts);
    }

    /**
     * 根据后续内容长度写入长度数据
     **/
    private static byte[] writeLen(int index, byte[] byts, ByteArrayOutputStream ms) {
        int len = byts.length - index;

        ms.reset();
        ms.write(byts, 0, index);
        writeLenByte(len, ms);
        ms.write(byts, index, len);

        return ms.toByteArray();
    }

    /**
     * 把字符串按每行多少个字断行
     **/
    private static String textBreak(String text, int line) {
        int idx = 0;
        int len = text.length();
        StringBuilder str = new StringBuilder();
        while (idx < len) {
            if (idx > 0) {
                str.append('\n');
            }
            if (idx + line >= len) {
                str.append(text.substring(idx));
            } else {
                str.append(text, idx, idx + line);
            }
            idx += line;
        }
        return str.toString();
    }

    /***
     * 将XML格式密钥转成PEM，支持公钥xml、私钥xml
     *  xml格式：
     *  <RSAKeyValue>
     *      <Modulus>modulus encode with base64</Modulus>
     *      <Exponent>publicExponent encode with base64</Exponent>
     *      <D>privateExponent encode with base64</D>
     *      <P>prime1 encode with base64</P>
     *      <Q>prime2 encode with base64</Q>
     *      <DP>exponent1 encode with base64</DP>
     *      <DQ>exponent2 encode with base64</DQ>
     *      <InverseQ>coefficient encode with base64</InverseQ>
     *  </RSAKeyValue>
     *
     */
    public static RSAKey fromXml(String xml) throws UnknownFormatConversionException {
        RSAKey rtv = new RSAKey();

        Matcher xmlM = XML_PATTERN.matcher(xml);
        if (!xmlM.find()) {
            throw new UnknownFormatConversionException("XML content does not meet requirements: there is no 'RSAKeyValue' tag or there is no content in the tag");
        }

        Matcher tagM = XML_TAG_PATTERN.matcher(xmlM.group(1));
        Base64.Decoder dec = Base64.getDecoder();
        while (tagM.find()) {
            String tag = tagM.group(1);
            String b64 = tagM.group(2);
            byte[] val = dec.decode(b64);
            switch (tag) {
                case "Modulus":
                    rtv.modulus = val;
                    break;
                case "Exponent":
                    rtv.publicExponent = val;
                    break;
                case "D":
                    rtv.privateExponent = val;
                    break;
                case "P":
                    rtv.prime1 = val;
                    break;
                case "Q":
                    rtv.prime2 = val;
                    break;
                case "DP":
                    rtv.exponent1 = val;
                    break;
                case "DQ":
                    rtv.exponent2 = val;
                    break;
                case "InverseQ":
                    rtv.coefficient = val;
                    break;
            }
        }

        if (rtv.modulus == null || rtv.publicExponent == null) {
            throw new UnknownFormatConversionException("XML public key missing");
        }
        if (rtv.privateExponent != null) {
            if (rtv.prime1 == null || rtv.prime2 == null || rtv.exponent1 == null || rtv.exponent2 == null || rtv.coefficient == null) {
                return new RSAKey(rtv.modulus, rtv.publicExponent, rtv.privateExponent);
            }
        }

        return rtv;
    }


    /***
     * 将RSA中的密钥对转换成XML格式
     * ，如果convertToPublic含私钥的RSA将只返回公钥，仅含公钥的RSA不受影响
     */
    public String toXml(boolean convertToPublic) {
        Base64.Encoder enc = Base64.getEncoder();
        StringBuilder str = new StringBuilder();
        str.append("<RSAKeyValue>");
        str.append("<Modulus>").append(enc.encodeToString(modulus)).append("</Modulus>");
        str.append("<Exponent>").append(enc.encodeToString(publicExponent)).append("</Exponent>");
        //  公钥没有后面部分
        if (this.privateExponent != null && !convertToPublic) {
            str.append("<P>").append(enc.encodeToString(prime1)).append("</P>");
            str.append("<Q>").append(enc.encodeToString(prime2)).append("</Q>");
            str.append("<DP>").append(enc.encodeToString(exponent1)).append("</DP>");
            str.append("<DQ>").append(enc.encodeToString(exponent2)).append("</DQ>");
            str.append("<InverseQ>").append(enc.encodeToString(coefficient)).append("</InverseQ>");
            str.append("<D>").append(enc.encodeToString(privateExponent)).append("</D>");
        }
        str.append("</RSAKeyValue>");
        return str.toString();
    }


    public RSAPublicKey toPublicKey() throws NoSuchAlgorithmException, InvalidKeySpecException {
        BigInteger m = new BigInteger(1, modulus);
        BigInteger e = new BigInteger(1, publicExponent);
        return (RSAPublicKey) KeyFactory.getInstance("RSA").generatePublic(new RSAPublicKeySpec(m, e));
    }

    public RSAPrivateKey toPrivateKey() throws NoSuchAlgorithmException, InvalidKeySpecException {
        RSAPrivateCrtKeySpec keySpec = new RSAPrivateCrtKeySpec(
                new BigInteger(1, modulus),
                new BigInteger(1, publicExponent),
                new BigInteger(1, privateExponent),
                new BigInteger(1, prime1),
                new BigInteger(1, prime2),
                new BigInteger(1, exponent1),
                new BigInteger(1, exponent2),
                new BigInteger(1, coefficient)
        );
        return (RSAPrivateKey) KeyFactory.getInstance("RSA").generatePrivate(keySpec);
    }
}
