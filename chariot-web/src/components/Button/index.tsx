import React from "react";
import styles from './Button.module.scss';

interface ButtonProps {
	text: string;
	onClick: () => void,
	state?: 'unselected' | 'selected' | 'voted';
	style?: 'boxy' | 'minimal',
	width?: string
}

export const Button: React.FC<ButtonProps> = ({ text, onClick, state = 'unselected', style = 'boxy', width }) => {
	return (
		<div
			className={
				`${styles.button} ${style === 'boxy' ? styles.full : ""} ${state === 'selected' ? styles.selected : (state === 'voted' ? styles.voted : '')}`
			}
			style={{
				width
			}}
			onClick={onClick}>
			{text}
		</div>
	)
}