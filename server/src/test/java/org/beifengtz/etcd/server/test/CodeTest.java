package org.beifengtz.etcd.server.test;

import org.junit.jupiter.api.Test;

import java.util.Random;

/**
 * description: TODO
 * date: 16:24 2023/12/7
 *
 * @author beifengtz
 */
public class CodeTest {

    @Test
    public void test() {
        String code = "beifengtz";
        long l = Long.parseLong(code, 36);
        System.out.println(l);

        Random random = new Random();
        int radix = random.nextInt(35) + 2;
        System.out.println(radix + " => " + Long.toString(l, radix));

        for (int i = 2; i <= 36; i++) {
            System.out.println(i + " => " + Long.toString(l, i));
        }
    }
}
