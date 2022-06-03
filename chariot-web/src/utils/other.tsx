export const toPercentage = (num: number): string => {
	return Math.floor(num * 100).toFixed(1) + "%"
}