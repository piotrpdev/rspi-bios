export function check<T>(arg: T): NonNullable<T> {
	if (arg == null) {
		throw new Error("check failed, arg is null/undefined");
	}
	return arg;
}

export function timeout(ms: number) {
	return new Promise((resolve) => setTimeout(resolve, ms));
}

export function showSession(id: string) {
	const session = check(document.getElementById(id));
	session.style.visibility = "visible";
}

export function exitBoot() {
	const startup = check(document.getElementById("startup"));
	startup.style.display = "none";
}
