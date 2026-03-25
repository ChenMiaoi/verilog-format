package net.ericsonj.verilog;

import java.util.Arrays;
import java.util.LinkedList;
import org.junit.Assert;
import org.junit.Test;

public class IndentationStyleTest {

    @Test
    public void alignsAlwaysEndWithAlwaysAndEndmoduleWithModule() {
        LinkedList<String> buffer = new LinkedList<>(Arrays.asList(
                "module demo;",
                "always @(posedge clk) begin",
                "data <= next_data;",
                "end",
                "endmodule"
        ));

        FileFormat format = new FileFormat(new FormatSetting(null));
        new IndentationStyle().applyStyle(format, buffer);

        Assert.assertEquals(Arrays.asList(
                "module demo;",
                "    always @(posedge clk) begin",
                "        data <= next_data;",
                "    end",
                "endmodule"
        ), buffer);
    }

    @Test
    public void exitsSingleLineAlwaysBeforeTheNextModuleLevelStatement() {
        LinkedList<String> buffer = new LinkedList<>(Arrays.asList(
                "module demo;",
                "always @(posedge clk)",
                "a <= b;",
                "always @(posedge clk)",
                "c <= d;",
                "endmodule // demo"
        ));

        FileFormat format = new FileFormat(new FormatSetting(null));
        new IndentationStyle().applyStyle(format, buffer);

        Assert.assertEquals(Arrays.asList(
                "module demo;",
                "    always @(posedge clk)",
                "        a <= b;",
                "    always @(posedge clk)",
                "        c <= d;",
                "endmodule // demo"
        ), buffer);
    }
}
