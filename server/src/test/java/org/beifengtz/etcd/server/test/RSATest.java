package org.beifengtz.etcd.server.test;

import org.beifengtz.jvmm.common.util.SignatureUtil;
import org.beifengtz.jvmm.common.util.meta.PairKey;
import org.junit.jupiter.api.Test;

/**
 * description: TODO
 * date: 15:45 2023/12/7
 *
 * @author beifengtz
 */
public class RSATest {
    @Test
    public void generatePairKey() throws Exception {
        PairKey<String, String> pairKey = SignatureUtil.genRSAKeyPair();
        System.out.println("public: " + pairKey.getLeft());
        System.out.println("private: " + pairKey.getRight());
    }
}
