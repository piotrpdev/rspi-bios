<script lang="ts">
import { onMount } from "svelte";
import { exitBoot, showSession, timeout } from "../../../helpers";

export let isFinished = false;

onMount(async () => {
	showSession("session-6");
	await timeout(500);
	showSession("session-7");
	await timeout(800);
	showSession("session-8");
	await timeout(500);
	showSession("session-9");
	await timeout(1200);
	showSession("session-10");
	await timeout(1000);
	exitBoot();
	isFinished = true;
});
</script>

<div id="session-6" style="visibility: hidden;">
    <br />
    <div style="text-align: left; background-color: black; ">
        <fieldset class="tui-fieldset" style="padding: 12px 0;">
            <table id="boot-table" class="tui-table" style="border: 0; width: 100%;">
                <tbody>
                      <tr>
                        <td>CPU Type</td>
                        <td>:</td>
                        <td>CORTEX-A</td>
                        <td>Base Memory</td>
                        <td>:</td>
                        <td>1048576K</td>
                      </tr>
                      <tr>
                        <td>Co-Processor</td>
                        <td>:</td>
                        <td>Installed</td>
                        <td>Extended Memory</td>
                        <td>:</td>
                        <td>3145728K</td>
                      </tr>
                      <tr>
                        <td>CPU Clock</td>
                        <td>:</td>
                        <td>1500MHz</td>
                        <td>Cache Memory</td>
                        <td>:</td>
                        <td>80K L1, 1.0M L2</td>
                      </tr>
                      <tr>
                        <td>Diskette Drive A</td>
                        <td>:</td>
                        <td>None</td>
                        <td>Display Type</td>
                        <td>:</td>
                        <td>HDMI</td>
                      </tr>
                      <tr>
                        <td>Diskette Drive B</td>
                        <td>:</td>
                        <td>None</td>
                        <td>Serial Port(s)</td>
                        <td>:</td>
                        <td>14 (TXD), 15 (RXD) </td>
                      </tr>
                      <tr>
                        <td>Pri. Master Disk</td>
                        <td>:</td>
                        <td>EEPROM ,Mode 2, 512KB</td>
                        <td>Parallel Port(s)</td>
                        <td>:</td>
                        <td>None</td>
                      </tr>
                      <tr>
                        <td>Pri. Slave Disk</td>
                        <td>:</td>
                        <td>SD ,Mode 4, 32GB</td>
                        <td>EDO DRAM at Row(s)</td>
                        <td>:</td>
                        <td>None</td>
                      </tr>
                      <tr>
                        <td>Sec. Master Disk</td>
                        <td>:</td>
                        <td>None</td>
                        <td>SDRAM at Row(s)</td>
                        <td>:</td>
                        <td>0 1 2 3 4</td>
                      </tr>
                      <tr>
                        <td>Sec. Slave Disk</td>
                        <td>:</td>
                        <td>None</td>
                        <td>L2 Cache Type</td>
                        <td>:</td>
                        <td>Data, Instruction</td>
                      </tr>
                </tbody>
            </table>
        </fieldset>
    </div>
    <br />
    <br />
    <br />
    <span id="session-7" style="visibility: hidden;">PCI device listing.....</span>
    <div id="session-8" style="text-align: left; background-color: black; visibility: hidden;">
        <fieldset class="tui-fieldset" style="padding: 12px 0;">
            <table id="pci-table" class="tui-table">
                <tbody>
                      <tr>
                        <td>Bus No.</td>
                        <td>Device No.</td>
                        <td>Func No.</td>
                        <td>Vendor ID</td>
                        <td>Device ID</td>
                        <td>Device Class</td>
                        <td>IRQ</td>
                      </tr>
                      <tr>
                        <td>0</td>
                        <td>7</td>
                        <td>1</td>
                        <td>0086</td>
                        <td>1230</td>
                        <td>IDE Controller</td>
                        <td>14</td>
                      </tr>
                      <tr>
                        <td>0</td>
                        <td>17</td>
                        <td>0</td>
                        <td>1274</td>
                        <td>1371</td>
                        <td>Multimedia Device</td>
                        <td>11</td>
                      </tr>
                </tbody>
            </table>
        </fieldset>
    </div>
    <br />
    <br />
    <span id="session-9" style="visibility: hidden;">Verifying DMI Pool Data ......</span>
    <br />
    <span id="session-10" style="visibility: hidden;">Starting Raspberry Pi OS.....</span>
</div>
