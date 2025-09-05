import React from 'react';
import ReactECharts from 'echarts-for-react';
import { Statistics, TimeFilter } from '../types';
import * as echarts from 'echarts';

interface StatisticsChartsProps {
  statistics: Statistics | null;
  filter?: TimeFilter;
}

const StatisticsCharts: React.FC<StatisticsChartsProps> = ({ statistics, filter }) => {
  const getWeeklyChartOption = () => {
    // 定义星期名称
    const weekdays = ['周日', '周一', '周二', '周三', '周四', '周五', '周六'];
    
    // 初始化每周的数据数组
    const weeklyData = Array(7).fill(0).map((_, i) => {
      const stat = statistics?.weekly.find(w => w.weekday === i);
      return {
        weekday: i,
        name: weekdays[i],
        additions: stat ? stat.additions : 0,
        deletions: stat ? stat.deletions : 0,
        commits: stat ? stat.commits : 0
      };
    });
    
    return {
      title: {
        text: '按星期代码变更分布',
        left: 'center'
      },
      tooltip: {
        trigger: 'axis',
        formatter: (params: any) => {
          const data = weeklyData[params[0].dataIndex];
          return `${data.name}<br/>` +
                 `新增: ${data.additions} 行<br/>` +
                 `删除: ${data.deletions} 行<br/>` +
                 `提交: ${data.commits} 次`;
        }
      },
      legend: {
        data: ['新增', '删除'],
        top: '10%'
      },
      xAxis: {
        type: 'category',
        data: weekdays
      },
      yAxis: {
        type: 'value',
        name: '代码行数'
      },
      series: [
        {
          name: '新增',
          data: weeklyData.map(d => d.additions),
          type: 'bar',
          stack: 'total',
          itemStyle: {
            color: '#28a745'
          }
        },
        {
          name: '删除',
          data: weeklyData.map(d => d.deletions),
          type: 'bar',
          stack: 'total',
          itemStyle: {
            color: '#dc3545'
          }
        }
      ]
    };
  };

  if (!statistics) {
    return (
      <div className="empty-state">
        <h3>暂无数据</h3>
        <p>请先添加仓库并刷新数据</p>
      </div>
    );
  }

  const getHourlyChartOption = () => {
    const hours = Array.from({ length: 24 }, (_, i) => i);
    const hourlyData = hours.map(hour => {
      const stat = statistics.hourly.find(h => h.hour === hour);
      return stat ? stat.additions + stat.deletions : 0;
    });

    return {
      title: {
        text: '24小时代码变更分布',
        left: 'center'
      },
      tooltip: {
        trigger: 'axis',
        formatter: (params: any) => {
          const hour = params[0].dataIndex;
          const stat = statistics.hourly.find(h => h.hour === hour);
          if (!stat) return `${hour}:00<br/>无数据`;
          return `${hour}:00<br/>` +
                 `新增: ${stat.additions} 行<br/>` +
                 `删除: ${stat.deletions} 行<br/>` +
                 `提交: ${stat.commits} 次`;
        }
      },
      xAxis: {
        type: 'category',
        data: hours.map(h => `${h}:00`),
        axisLabel: {
          interval: 1
        }
      },
      yAxis: {
        type: 'value',
        name: '代码行数'
      },
      series: [{
        data: hourlyData,
        type: 'bar',
        itemStyle: {
          color: '#007acc'
        }
      }]
    };
  };

  const getDailyChartOption = () => {
    // 计算时间范围天数
    let days = 7; // 默认7天
    if (filter?.start_date) {
      const startDate = new Date(filter.start_date);
      const endDate = filter?.end_date ? new Date(filter.end_date) : new Date();
      // 计算天数差
      const diffTime = Math.abs(endDate.getTime() - startDate.getTime());
      days = Math.ceil(diffTime / (1000 * 60 * 60 * 24)) + 1; // 包含起始和结束日期
    } else if (filter?.end_date) {
      // 如果只有结束日期，计算到今天的天数
      const endDate = new Date(filter.end_date);
      const today = new Date();
      const diffTime = Math.abs(today.getTime() - endDate.getTime());
      days = Math.ceil(diffTime / (1000 * 60 * 60 * 24));
    }
    
    // 生成日期列表
    const dates = [];
    const endDate = new Date();
    for (let i = days - 1; i >= 0; i--) {
      const date = new Date(endDate);
      date.setDate(endDate.getDate() - i);
      const dateString = date.toISOString().split('T')[0];
      dates.push(dateString);
    }
    
    // 为每个日期准备数据，如果没有记录则使用默认值
    const chartData = dates.map(date => {
      const stat = statistics.daily.find(d => d.date === date);
      return {
        date,
        additions: stat ? stat.additions : 0,
        deletions: stat ? stat.deletions : 0,
        commits: stat ? stat.commits : 0
      };
    });
    
    // 动态标题
    const title = days <= 1 ? '按天代码变更趋势' : `最近${days}天代码变更趋势`;
    
    return {
      title: {
        text: title,
        left: 'center'
      },
      tooltip: {
        trigger: 'axis',
        formatter: (params: any) => {
          const data = chartData[params[0].dataIndex];
          return `${data.date}<br/>` +
                 `新增: ${data.additions} 行<br/>` +
                 `删除: ${data.deletions} 行<br/>` +
                 `提交: ${data.commits} 次`;
        }
      },
      legend: {
        data: ['新增', '删除'],
        top: '10%'
      },
      xAxis: {
        type: 'category',
        data: chartData.map(d => d.date.substring(5)) // MM-DD format
      },
      yAxis: {
        type: 'value',
        name: '代码行数'
      },
      series: [
        {
          name: '新增',
          data: chartData.map(d => d.additions),
          type: 'bar',
          stack: 'total',
          itemStyle: {
            color: '#28a745'
          }
        },
        {
          name: '删除',
          data: chartData.map(d => d.deletions),
          type: 'bar',
          stack: 'total',
          itemStyle: {
            color: '#dc3545'
          }
        }
      ]
    };
  };

  const getAuthorChartOption = () => {
    const authors = Object.entries(statistics.authors).slice(0, 10); // Top 10 authors
    
    return {
      title: {
        text: 'TOP 10 贡献者',
        left: 'center'
      },
      tooltip: {
        trigger: 'item',
        formatter: (params: any) => {
          const [name, data] = authors[params.dataIndex];
          return `${name}<br/>` +
                 `新增: ${data.additions} 行<br/>` +
                 `删除: ${data.deletions} 行<br/>` +
                 `提交: ${data.commits} 次`;
        }
      },
      series: [{
        type: 'pie',
        radius: '50%',
        data: authors.map(([name, data]) => ({
          name,
          value: data.additions + data.deletions
        })),
        emphasis: {
          itemStyle: {
            shadowBlur: 10,
            shadowOffsetX: 0,
            shadowColor: 'rgba(0, 0, 0, 0.5)'
          }
        }
      }]
    };
  };

  const getRepositoryChartOption = () => {
    const repos = Object.entries(statistics.repositories);
    
    return {
      title: {
        text: '仓库贡献分布',
        left: 'center'
      },
      tooltip: {
        trigger: 'item',
        formatter: (params: any) => {
          const [name, data] = repos[params.dataIndex];
          return `${name}<br/>` +
                 `新增: ${data.additions} 行<br/>` +
                 `删除: ${data.deletions} 行<br/>` +
                 `提交: ${data.commits} 次`;
        }
      },
      series: [{
        type: 'pie',
        radius: ['40%', '70%'],
        data: repos.map(([name, data]) => ({
          name: name.split('/').pop() || name, // 只显示仓库名
          value: data.additions + data.deletions
        })),
        emphasis: {
          itemStyle: {
            shadowBlur: 10,
            shadowOffsetX: 0,
            shadowColor: 'rgba(0, 0, 0, 0.5)'
          }
        }
      }]
    };
  };

  // Calendar view for last 30 days
  const getCalendarChartOption = () => {
    // Get last 30 days data
    const dailyData = statistics.daily.slice(0, 30); // First 30 items (most recent)
    const calendarData = dailyData.map(d => [d.date, d.additions + d.deletions]);
    
    // Calculate date range for last 30 days
    const endDate = dailyData.length > 0 ? new Date(dailyData[0].date) : new Date();
    const startDate = new Date(endDate);
    startDate.setDate(startDate.getDate() - 29); // 30 days including today
    
    // Format dates as strings for ECharts
    const startDateStr = startDate.toISOString().split('T')[0];
    const endDateStr = endDate.toISOString().split('T')[0];

    return {
      title: {
        text: '最近30天代码变更日历',
        left: 'center'
      },
      tooltip: {
        position: 'top',
        formatter: (params: any) => {
          const [date, value] = params.data;
          const dateObj = new Date(date);
          const year = dateObj.getFullYear();
          const month = String(dateObj.getMonth() + 1).padStart(2, '0');
          const day = String(dateObj.getDate()).padStart(2, '0');
          const formattedDate = `${year}-${month}-${day}`;
          
          // Find the original daily stat for this date
          const stat = dailyData.find(d => d.date === formattedDate);
          
          if (!stat) return `${formattedDate}<br/>无数据`;
          
          return `${formattedDate}<br/>` +
                 `新增: ${stat.additions} 行<br/>` +
                 `删除: ${stat.deletions} 行<br/>` +
                 `提交: ${stat.commits} 次<br/>` +
                 `总计: ${value} 行`;
        }
      },
      visualMap: {
        min: 0,
        max: Math.max(1, ...calendarData.map(item => item[1] as number)), // Ensure max is at least 1
        calculable: true,
        orient: 'horizontal',
        left: 'center',
        bottom: '0%',
        inRange: {
          color: ['#ebedf0', '#c6e48b', '#7bc96f', '#239a3b', '#196127']
        }
      },
      calendar: {
        top: 'middle',
        left: 'center',
        orient: 'horizontal',
        cellSize: ['auto', 25], // Increased cell size
        range: [startDateStr, endDateStr],
        itemStyle: {
          borderWidth: 1
        },
        yearLabel: { show: true },
        dayLabel: {
          firstDay: 1, // Start week on Monday
          margin: 5
        },
        monthLabel: {
          margin: 5
        }
      },
      series: {
        type: 'heatmap',
        coordinateSystem: 'calendar',
        data: calendarData
      }
    };
  };

  return (
    <div className="charts-container">
      <div className="chart-card">
        <ReactECharts
          option={getHourlyChartOption()}
          style={{ height: '300px' }}
          notMerge={true}
        />
      </div>
      
      <div className="chart-card">
        <ReactECharts
          option={getDailyChartOption()}
          style={{ height: '300px' }}
          notMerge={true}
        />
      </div>
      
      <div className="chart-card">
        <ReactECharts
          option={getWeeklyChartOption()}
          style={{ height: '300px' }}
          notMerge={true}
        />
      </div>
      
      <div className="chart-card">
        <ReactECharts
          option={getAuthorChartOption()}
          style={{ height: '300px' }}
          notMerge={true}
        />
      </div>
      
      <div className="chart-card">
        <ReactECharts
          option={getRepositoryChartOption()}
          style={{ height: '300px' }}
          notMerge={true}
        />
      </div>
      
      {/* Calendar Heatmap Chart */}
      <div className="chart-card">
        <ReactECharts
          option={getCalendarChartOption()}
          style={{ height: '300px' }}
          notMerge={true}
        />
      </div>
    </div>
  );
};

export default StatisticsCharts;