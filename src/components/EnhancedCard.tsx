import React from 'react';
import { Card } from 'antd';
import { CardProps } from 'antd/es/card';

interface EnhancedCardProps extends CardProps {
  gradient?: boolean;
  glow?: boolean;
}

const EnhancedCard: React.FC<EnhancedCardProps> = ({ 
  children, 
  gradient = false, 
  glow = false,
  className = '',
  ...props 
}) => {
  const classes = [
    className,
    'hover-lift',
    gradient ? 'gradient-border' : '',
    glow ? 'glow-effect' : '',
  ].filter(Boolean).join(' ');

  return (
    <Card className={classes} {...props}>
      {children}
    </Card>
  );
};

export default EnhancedCard;




