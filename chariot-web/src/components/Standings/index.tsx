import React, { useContext } from 'react';
import { GlobalContext } from '../../contexts/GlobalContext';
import styles from './Standings.module.scss'

const Standings: React.FC = () => {
	const { standings } = useContext(GlobalContext);
	return <table className={styles.table}>
		<th>
			<td>player</td>
			<td>rank</td>
			<td>lap</td>
			<td>chair</td>
		</th>
		{standings.map((standing) => (
			<tr className={styles.row} key={standing.name}>
				<td>{standing.name}</td>
				<td>{standing.rank}</td>
				<td>{standing.lap}</td>
				<td>{standing.chair}</td>
			</tr>
		))}
	</table>
}

export default Standings;