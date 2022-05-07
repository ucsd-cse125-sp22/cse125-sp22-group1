import React from "react";
import styles from './Button.module.scss';

interface ButtonProps {
	text: string;
	state?: 'unselected' | 'selected' | 'voted';
	style?: 'boxy' | 'minimal'
	onClick: () => void
}

export const Button: React.FC<ButtonProps> = ({ text, onClick, state = 'unselected', style = 'boxy' }) => {
	return <div className={`${styles.button} ${style === 'boxy' ? styles.full : ""}`} onClick={onClick}>
		{text}
	</div>
}