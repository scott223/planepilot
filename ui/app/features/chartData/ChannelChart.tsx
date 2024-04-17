import React from 'react'
import { useSelector, useDispatch } from 'react-redux'
import { fetchDataRequest } from './actions'
import { chartStateType } from './types'

interface DataProps {
    channel: number,
}

export const ChannelChart: React.FC<DataProps> = props => {
    const { channel } = props;
    const chartData = useSelector((state: chartStateType) => state.chartData)
    const dispatch = useDispatch()

    return (
        <div>
            <div>
                <button
                    aria-label="Add chart data"
                    onClick={() => dispatch(fetchDataRequest(channel))}
                >
                    Add chart data
                </button>
            </div>
        </div>
    )
}