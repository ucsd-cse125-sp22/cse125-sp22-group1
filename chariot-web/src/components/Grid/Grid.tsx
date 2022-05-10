import React, { PropsWithChildren } from 'react';
import styles from './Grid.module.scss';

const Grid: React.FC<PropsWithChildren<{}>> = ({ children }) => {
	return <div className={styles.grid}>{children}</div>
}

export default Grid;