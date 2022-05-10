import React from "react";
import styles from './Button.module.scss';

interface ButtonProps {
	text: string;
	onClick: () => void,
	state?: 'unselected' | 'selected' | 'voted';
	style?: 'boxy' | 'minimal'
}

export const Button: React.FC<ButtonProps> = ({ text, onClick, state = 'unselected', style = 'boxy' }) => {
	return (
		<div
			className={
				`${styles.button} ${style === 'boxy' ? styles.full : ""} ${state === 'selected' ? styles.selected : (state === 'voted' ? styles.voted : '')}`
			}
			onClick={onClick}>
			{text}
		</div>
	)
}