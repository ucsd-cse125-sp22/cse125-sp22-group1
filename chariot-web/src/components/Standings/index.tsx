import React, { useContext } from 'react';
import { GlobalContext } from '../../contexts/GlobalContext';
import styles from './Standings.module.scss'

const Standings: React.FC = () => {
	const { standings } = useContext(GlobalContext);
	return <table className={styles.table}>
		<tr>
			<th>player</th>
			<th>rank</th>
			<th>lap</th>
			<th>chair</th>
		</tr>
		{standings.map((standing) => (
			<tr key={standing.name}>
				<td>{standing.name}</td>
				<td>{standing.rank}</td>
				<td>{standing.lap}</td>
				<td>{standing.chair}</td>
			</tr>
		))}
	</table>
}

export default Standings;