import React from 'react';
import styles from './Standings.module.scss'

const Standings: React.FC = () => {
	return <table className={styles.table}>
		<tr>
			<th>player</th>
			<th>rank</th>
			<th>chair</th>
		</tr>
		<tr>
			<td>3</td>
			<td>1</td>
			<td>the classic</td>
		</tr>
	</table>
}

export default Standings;