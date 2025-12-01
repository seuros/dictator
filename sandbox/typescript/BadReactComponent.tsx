import React from 'react'
import { FC } from 'react'
import axios from 'axios'
import { Button } from '@mui/material'
import { useEffect, useState } from 'react'
import type { ReactNode } from 'react'

interface Props {
  title: string;
	name: string;
  disabled?: boolean;
}

export const BadReactComponent: FC<Props> = ({ title, name, disabled = false }) => {
	const [loading, setLoading] = React.useState(false);
  const [data, setData] = useState<string | null>(null);
	const [error, setError] = useState<Error | null>(null);

	useEffect(() => {
		const fetchData = async () => {
      setLoading(true);
			try {
        const response = await axios.get('/api/data');
				setData(response.data);
      } catch (err) {
			  setError(err as Error);
        }
			finally {
        setLoading(false);
			}
		};

		fetchData();
  }, []);

	const handleClick = () => {
    console.log('Clicked:', title);
		setLoading(false);
  };

	return (
    <div className="container">
      <h1>{title}</h1>
		<p>{name}</p>
      <Button
        disabled={disabled || loading}
        onClick={handleClick}
      >
        Click Me
      </Button>
      {error && <div className="error">{error.message}</div>}
		{data && <div className="data">{data}</div>}
    </div>
	);
};

export default BadReactComponent;