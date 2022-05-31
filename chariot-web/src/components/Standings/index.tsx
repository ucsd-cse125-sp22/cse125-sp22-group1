import React, { useContext } from 'react';
import { GlobalContext } from '../../contexts/GlobalContext';
import styles from './Standings.module.scss'

const Standings: React.FC = () => {
	const { standings } = useContext(GlobalContext);
	return (
		<div className={styles['table-container']} role="table" aria-label="Destinations">
			<div className={[styles["div"], styles["flex-table"]].join(" ")} role="rowgroup">
				<div className={[styles["div"], styles["flex-row"], styles["first"]].join(" ")} role="columnheader">Chair</div>
				<div className={[styles["div"], styles["flex-row"]].join(" ")} role="columnheader">Rank</div>
				<div className={[styles["div"], styles["flex-row"]].join(" ")} role="columnheader">Lap</div>
			</div>
			{standings.map(standing => {
				return (
					<div key={standing.name} className={[styles["div"], styles["flex-table"]].join(" ")} role="rowgroup">
						<div className={[styles["div"], styles["flex-row"], styles["first"]].join(" ")} role="cell">{standing.chair}</div>
						<div className={[styles["div"], styles["flex-row"]].join(" ")} role="cell">{standing.rank}</div>
						<div className={[styles["div"], styles["flex-row"]].join(" ")} role="cell">{standing.lap}</div>
					</div>
				)
			})}
		</div>
	)
}

export default Standings;