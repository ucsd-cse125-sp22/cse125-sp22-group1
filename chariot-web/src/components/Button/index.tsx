import React from "react";
import styles from './Button.module.scss';

interface ButtonProps {
	text: string;
	onClick: () => void,
	state?: 'unselected' | 'selected' | 'voted';
	style?: 'boxy' | 'minimal',
	clickable?: boolean,
	width?: string
}

export const Button: React.FC<ButtonProps> = ({ text, onClick, state = 'unselected', style = 'boxy', width, clickable = true }) => {
	` ${state === 'selected' ? styles.selected : (state === 'voted' ? styles.voted : '')}`
	return (
		<div
			className={
				[styles.button, ...clickable ? [styles.clickable] : [], ...style === 'boxy' ? [styles.full] : [], ...state === 'selected' ? [styles.selected] : (state === 'voted' ? [styles.voted] : [])].join(" ")

			}
			style={{
				width
			}}
			onClick={onClick}>
			{text}
		</div>
	)
}