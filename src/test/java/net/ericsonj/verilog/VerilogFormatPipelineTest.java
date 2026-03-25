package net.ericsonj.verilog;

import java.io.File;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import org.junit.Assert;
import org.junit.Test;

public class VerilogFormatPipelineTest {

    @Test
    public void formatsControlFlowAssignmentsAndCommentsUsingTheDefaultPipeline() {
        Assert.assertEquals(Arrays.asList(
                "module demo;",
                "    always @(posedge clk) begin",
                "        state_wait <= IDLE;",
                "        state      <= RUN; //next",
                "        if (done) begin",
                "            counter    = 0;",
                "            value_long = 1; //set",
                "        end",
                "        else if (error)",
                "            state <= FAIL;",
                "    end",
                "    assign short              = 1;",
                "    assign much_longer_signal = 2; //tail",
                "endmodule"
        ), VerilogFormatterTestHelper.formatLines(
                "module demo;",
                "always @(posedge clk)begin",
                "state_wait<=IDLE;",
                "state<=RUN;//next",
                "if(done)begin",
                "counter=0;",
                "value_long=1;//set",
                "end",
                "else if(error)",
                "state<=FAIL;",
                "end",
                "assign short=1;",
                "assign much_longer_signal=2;//tail",
                "endmodule"
        ));
    }

    @Test
    public void formatsCaseBlocksAndEndcaseIndentation() {
        Assert.assertEquals(Arrays.asList(
                "module demo;",
                "    always @(posedge clk) begin",
                "        case(state)",
                "            IDLE: begin",
                "                data <= 0;",
                "            end",
                "            default: begin",
                "                data <= 1;",
                "            end",
                "        endcase",
                "    end",
                "endmodule"
        ), VerilogFormatterTestHelper.formatLines(
                "module demo;",
                "always @(posedge clk) begin",
                "case(state)",
                "IDLE: begin",
                "data<=0;",
                "end",
                "default: begin",
                "data<=1;",
                "end",
                "endcase",
                "end",
                "endmodule"
        ));
    }

    @Test
    public void formatsModuleDefinitionsAcrossMultipleLines() {
        Assert.assertEquals(Arrays.asList(
                "module demo(",
                "    input clk,",
                "    input rst_n,",
                "    output reg data",
                ");",
                "endmodule"
        ), VerilogFormatterTestHelper.formatLines(
                "module demo(input clk,input rst_n,output reg data);",
                "endmodule"
        ));
    }

    @Test
    public void honorsSettingsForParenthesesSquareBracketsAndCommentAlignment() throws Exception {
        Path settingsFile = Files.createTempFile("verilog-format", ".properties");
        Files.write(settingsFile, Arrays.asList(
                "SpacesInParentheses=true",
                "SpacesInSquareBrackets=true",
                "AlignLineComments=true",
                "SpacesAfterTrailingComments=1"
        ), StandardCharsets.UTF_8);

        Assert.assertEquals(Arrays.asList(
                "module demo;",
                "    always @( posedge clk ) begin // a",
                "        data <= rom[ addr ];      // b",
                "    end",
                "endmodule"
        ), VerilogFormatterTestHelper.formatLines(
                settingsFile.toFile(),
                "module demo;",
                "always @(posedge clk) begin //a",
                "data<=rom[addr];//b",
                "end",
                "endmodule"
        ));
    }

    @Test
    public void formatsAndOverwritesFilesThroughVerilogFile() throws Exception {
        Path source = Files.createTempFile("verilog-format-source", ".v");
        Files.write(source, Arrays.asList(
                "  module demo;  ",
                " always @(posedge clk)",
                " a<=b; ",
                "endmodule "
        ), StandardCharsets.UTF_8);

        VerilogFile file = VerilogFormatterTestHelper.formatFile(source.toString(), (File) null);
        file.overWrite();

        Assert.assertEquals(Arrays.asList(
                "module demo;",
                "    always @(posedge clk)",
                "        a <= b;",
                "endmodule"
        ), Files.readAllLines(source, StandardCharsets.UTF_8));
    }
}
