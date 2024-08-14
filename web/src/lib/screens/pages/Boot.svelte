<script lang="ts">
import { onMount } from "svelte";
import { check, showSession, timeout } from "../../../helpers";
import awardSoftwareLogo from "../../../assets/award_software_logo.png";
import energyStarLogo from "../../../assets/energy_star_logo.png";

export let isFinished = false;

async function memoryTest() {
	// 4194304 OK
	const memoryTest = check(document.getElementById("memoryTest"));

	for (let memory = 0; memory < 4194304; memory += 41943) {
		memoryTest.innerHTML = `Memory Test : &nbsp;&nbsp; ${memory}K`;
		await timeout(1);
	}

	memoryTest.innerHTML = "Memory Test : &nbsp;&nbsp; 4194304K OK";
}

onMount(async () => {
	await memoryTest();
	await timeout(300);
	showSession("session-1");
	await timeout(100);
	showSession("session-2");
	await timeout(50);
	showSession("session-3");
	await timeout(420);
	showSession("session-4");
	await timeout(800);
	check(document.getElementById("session-0")).style.display = "none";
	await timeout(500);
	isFinished = true;
});
</script>

<div id="session-0">
    <img alt="Award logo" src={awardSoftwareLogo} width="40px" height="40px" style="float: left" />
    <img alt="Energy Star logo" src={energyStarLogo} width="200px" height="150px" style="float: right" />
    <div style="margin-top: 4px"></div>
    Raspberry Pi Kernel v6.1, An Energy Star Ally<br>
    Copyright (C) 2012-2023, Raspberry Pi Foundation & Broadcom.<br><br>
    Raspberry Pi 4 Model B Rev 1.4 Kernel Version 6.1<br><br>
    Broadcom(R) BCM2711 Cortex-A72 (4) 1500 MHz<br>
    <span id="memoryTest">Memory Test : &nbsp;&nbsp;</span>
    <br><br>
    <div id="session-1" style="visibility: hidden">
        Award Plug and Play BIOS Extension v1.0A<br>
        Initialize Plug and Play Cards...<br>
        PNP init Completed<br><br>
    </div>
    <div id="session-2" style="visibility: hidden">
        Detecting Primary Master .....: Boot EEPROM<br>
    </div>
    <div id="session-3" style="visibility: hidden">
        Detecting Primary Slave ......: SanDisk SD<br>
    </div>
    <div id="session-4" style="visibility: hidden">
        Detecting Secondary Master ...: Skip<br>
        Detecting Secondary Slave ....: None
    </div>
    <div class="tui-statusbar absolute black white-text">
        <ul>
            <li style="margin-left: 0px">Press <b>DEL</b> to enter SETUP, <b>Alt+F2</b> to enter EZ flash utility
            </li>
        </ul>
        <ul>
            <li style="margin-left: 0px">12/05/2023-04/BCM2711/RPI4B-UEFIv1.2</li>
        </ul>
    </div>
</div>