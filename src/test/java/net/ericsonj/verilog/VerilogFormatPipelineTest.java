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
    public void formatsModuleInstantiationsThroughTheDefaultPipeline() {
        Assert.assertEquals(Arrays.asList(
                "module demo;",
                "    pos_buffer pos_bufferEx01(",
                "        .fifo_rdreq  (fifo_rdreq),",
                "        .fifo_rddata (fifo_rddata),",
                "        .fifo_usedw  (fifo_usedw)",
                "    );",
                "endmodule"
        ), VerilogFormatterTestHelper.formatLines(
                "module demo;",
                "pos_buffer pos_bufferEx01(",
                ".fifo_rdreq(fifo_rdreq),",
                ".fifo_rddata(fifo_rddata),",
                ".fifo_usedw(fifo_usedw)",
                ");",
                "endmodule"
        ));
    }

    @Test
    public void formatsCaseStatementsWhoseLabelsComeFromParameters() {
        Assert.assertEquals(Arrays.asList(
                "module Fibnoacci(",
                "    input clk,",
                "    input reset,",
                "    input in_valid,",
                "    input [7:0]in_level,",
                "    output reg out_valid,",
                "    output reg [7:0]result",
                ");",
                "    ",
                "    parameter [1:0] IDLE = 0, CALCULATE = 1, COMPLETE = 2;",
                "    reg [1:0]Q, Q_NEXT;",
                "    ",
                "    always @(*) begin",
                "        case (Q)",
                "            IDLE:",
                "                if (in_valid)",
                "                    Q_NEXT = CALCULATE;",
                "                else",
                "                    Q_NEXT = IDLE;",
                "            CALCULATE:",
                "                if (done)",
                "                    Q_NEXT = COMPLETE;",
                "                else",
                "                    Q_NEXT = CALCULATE;",
                "            COMPLETE:",
                "                Q_NEXT      = IDLE;",
                "            default: Q_NEXT = IDLE;",
                "        endcase",
                "    end",
                "endmodule // Fibnoacci"
        ), VerilogFormatterTestHelper.formatLines(
                "module Fibnoacci(input clk,",
                "input reset,",
                "input in_valid,",
                "input [7:0]in_level,",
                "output reg out_valid,",
                "output reg [7:0]result);",
                "",
                "parameter [1:0] IDLE = 0, CALCULATE = 1, COMPLETE = 2;",
                "reg [1:0]Q, Q_NEXT;",
                "",
                "always @(*) begin",
                "case (Q)",
                "IDLE:",
                "if (in_valid)",
                "Q_NEXT = CALCULATE;",
                "else",
                "Q_NEXT = IDLE;",
                "CALCULATE:",
                "if (done)",
                "Q_NEXT = COMPLETE;",
                "else",
                "Q_NEXT = CALCULATE;",
                "COMPLETE:",
                "Q_NEXT      = IDLE;",
                "default: Q_NEXT = IDLE;",
                "endcase",
                "end",
                "endmodule // Fibnoacci"
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
