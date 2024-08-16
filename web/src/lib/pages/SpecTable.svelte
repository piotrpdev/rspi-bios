<script lang="ts">
import { timeout } from "@lib/helpers";
import { onMount } from "svelte";

export let isFinished = false;
let showPciWait = false;
let showPciTable = false;
let showDmi = false;
let showStart = false;

onMount(async () => {
	await timeout(500);
	showPciWait = true;
	await timeout(800);
	showPciTable = true;
	await timeout(500);
	showDmi = true;
	await timeout(1200);
	showStart = true;
	await timeout(1000);
	isFinished = true;
});
</script>

<div>
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
    {#if showPciWait}
      <span>PCI device listing.....</span>
    {/if}
    {#if showPciTable}
      <div style="text-align: left; background-color: black;">
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
    {/if}
    <br />
    <br />
    {#if showDmi}
      <span id="session-9">Verifying DMI Pool Data ......</span>
    {/if}
    <br />
    {#if showStart}
      <span id="session-10">Starting Raspberry Pi OS.....</span>
    {/if}
</div>
