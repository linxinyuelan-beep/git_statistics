import React from 'react';
import ReactECharts from 'echarts-for-react';
import 'echarts-wordcloud'; // Import wordcloud extension
import { Statistics, TimeFilter } from '../types';

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
    let startDateObj;
    let endDateObj;
    
    if (filter?.start_date) {
      startDateObj = new Date(filter.start_date);
      endDateObj = filter?.end_date ? new Date(filter.end_date) : new Date();
      // 计算天数差
      const diffTime = Math.abs(endDateObj.getTime() - startDateObj.getTime());
      days = Math.ceil(diffTime / (1000 * 60 * 60 * 24)) + 1; // 包含起始和结束日期
    } else if (filter?.end_date) {
      // 如果只有结束日期，计算从end_date减去days天的范围
      endDateObj = new Date(filter.end_date);
      startDateObj = new Date(endDateObj);
      startDateObj.setDate(endDateObj.getDate() - days + 1); // 向前推7天
    } else {
      // 默认情况：最近7天
      endDateObj = new Date();
      startDateObj = new Date(endDateObj);
      startDateObj.setDate(endDateObj.getDate() - days + 1);
    }
    
    // 生成日期列表 - 使用实际的开始和结束日期
    const dates = [];
    let currentDate = new Date(startDateObj);
    
    // 循环从开始日期到结束日期
    while (currentDate <= endDateObj) {
      const dateString = currentDate.toISOString().split('T')[0];
      dates.push(dateString);
      currentDate.setDate(currentDate.getDate() + 1);
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
    // Sort authors by total contributions (additions + deletions) in descending order
    const sortedAuthors = Object.entries(statistics.authors)
      .sort((a, b) => (b[1].additions + b[1].deletions) - (a[1].additions + a[1].deletions))
      .slice(0, 10); // Top 10 authors
    
    return {
      title: {
        text: 'TOP 10 贡献者',
        left: 'center'
      },
      tooltip: {
        trigger: 'item',
        formatter: (params: any) => {
          const [name, data] = sortedAuthors[params.dataIndex];
          return `${name}<br/>` +
                 `新增: ${data.additions} 行<br/>` +
                 `删除: ${data.deletions} 行<br/>` +
                 `提交: ${data.commits} 次`;
        }
      },
      series: [{
        type: 'pie',
        radius: '50%',
        data: sortedAuthors.map(([name, data]) => ({
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
    // Sort repositories by total contributions (additions + deletions) in descending order
    const sortedRepos = Object.entries(statistics.repositories)
      .sort((a, b) => (b[1].additions + b[1].deletions) - (a[1].additions + a[1].deletions));
    
    return {
      title: {
        text: '仓库贡献分布',
        left: 'center'
      },
      tooltip: {
        trigger: 'item',
        formatter: (params: any) => {
          const [name, data] = sortedRepos[params.dataIndex];
          return `${name}<br/>` +
                 `新增: ${data.additions} 行<br/>` +
                 `删除: ${data.deletions} 行<br/>` +
                 `提交: ${data.commits} 次`;
        }
      },
      series: [{
        type: 'pie',
        radius: ['40%', '70%'],
        data: sortedRepos.map(([name, data]) => ({
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

  // Calendar view for the selected date range or last 30 days
  const getCalendarChartOption = () => {
    let startDate, endDate;
    let daysRange = 30; // 默认30天
    let titleText = '所选时间范围内代码变更日历';
    
    // 使用过滤器中的日期范围（如果有）
    if (filter?.start_date) {
      startDate = new Date(filter.start_date);
      endDate = filter?.end_date ? new Date(filter.end_date) : new Date();
      const diffDays = Math.ceil((endDate.getTime() - startDate.getTime()) / (1000 * 60 * 60 * 24)) + 1;
      daysRange = diffDays;
      titleText = `${daysRange}天代码变更日历`;
    } else {
      // 默认显示最近30天
      endDate = new Date();
      startDate = new Date(endDate);
      startDate.setDate(startDate.getDate() - 29); // 30天（包括今天）
    }
    
    // 格式化日期为字符串
    const startDateStr = startDate.toISOString().split('T')[0];
    const endDateStr = endDate.toISOString().split('T')[0];
    
    // 获取日期范围内的所有数据
    // statistics.daily现在应该包含所有日期的数据，不限于30天
    const dailyData = statistics.daily.filter(d => {
      const date = new Date(d.date);
      return date >= startDate && date <= endDate;
    });
    
    const calendarData = dailyData.map(d => [d.date, d.additions + d.deletions]);

    return {
      title: {
        text: titleText,
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

  // Hourly commit distribution heatmap (hour x day of week)
  const getHourlyCommitDistributionChartOption = () => {
    // Create a 24x7 matrix for the heatmap
    const hours = Array.from({ length: 24 }, (_, i) => i);
    const days = Array.from({ length: 7 }, (_, i) => i);
    const weekdays = ['周日', '周一', '周二', '周三', '周四', '周五', '周六'];
    
    // Prepare data for heatmap
    const heatmapData = [];
    for (const hour of hours) {
      for (const day of days) {
        const stat = statistics.hourly_commit_distribution.find(
          h => h.hour === hour && h.day_of_week === day
        );
        const commitCount = stat ? stat.commits : 0;
        heatmapData.push([hour, day, commitCount]);
      }
    }
    
    return {
      title: {
        text: '代码提交活跃时段热力图',
        left: 'center'
      },
      tooltip: {
        position: 'top',
        formatter: (params: any) => {
          const [hour, day, commits] = params.data;
          return `${weekdays[day]} ${hour}:00-${hour + 1}:00<br/>` +
                 `提交次数: ${commits} 次`;
        }
      },
      grid: {
        height: '50%',
        top: '20%'
      },
      xAxis: {
        type: 'category',
        data: hours.map(h => `${h}:00`),
        splitArea: {
          show: true
        },
        axisLabel: {
          interval: 1
        }
      },
      yAxis: {
        type: 'category',
        data: weekdays,
        splitArea: {
          show: true
        }
      },
      visualMap: {
        min: 0,
        max: Math.max(1, ...heatmapData.map(item => item[2])), // Ensure max is at least 1
        calculable: true,
        orient: 'horizontal',
        left: 'center',
        bottom: '15%',
        inRange: {
          color: ['#ebedf0', '#c6e48b', '#7bc96f', '#239a3b', '#196127']
        }
      },
      series: [{
        name: '提交次数',
        type: 'heatmap',
        data: heatmapData,
        label: {
          show: false
        },
        emphasis: {
          itemStyle: {
            shadowBlur: 10,
            shadowColor: 'rgba(0, 0, 0, 0.5)'
          }
        }
      }]
    };
  };

  // Author activity trends
  const getAuthorActivityTrendsChartOption = () => {
    // Get unique authors and periods
    const authors = Array.from(new Set(statistics.author_activity_trends.map(a => a.author)));
    const periods = Array.from(new Set(statistics.author_activity_trends.map(a => a.period))).sort();
    
    // Only show top 10 most active authors to avoid cluttering the chart
    const authorStats = authors.map(author => {
      const authorData = statistics.author_activity_trends.filter(a => a.author === author);
      const totalCommits = authorData.reduce((sum, a) => sum + a.commits, 0);
      return { author, totalCommits };
    }).sort((a, b) => b.totalCommits - a.totalCommits).slice(0, 10);
    
    const topAuthors = authorStats.map(s => s.author);
    
    // Create series for each top author
    const series = topAuthors.map(author => {
      const authorData = statistics.author_activity_trends.filter(a => a.author === author);
      const data = periods.map(period => {
        const stat = authorData.find(a => a.period === period);
        return stat ? stat.commits : null; // Use null for missing data points
      });
      
      return {
        name: author,
        type: 'line',
        data: data,
        connectNulls: false,
        showSymbol: false, // Hide data points to reduce clutter
        lineStyle: {
          width: 2
        }
      };
    });
    
    // Limit the number of x-axis labels to prevent overcrowding
    const maxLabels = 30;
    let xAxisData = periods;
    let interval = 0;
    
    if (periods.length > maxLabels) {
      interval = Math.floor(periods.length / maxLabels);
      xAxisData = periods.filter((_, i) => i % (interval + 1) === 0);
    }
    
    return {
      title: {
        text: '贡献者活跃度趋势 (Top 10)',
        left: 'center'
      },
      tooltip: {
        trigger: 'axis',
        formatter: (params: any) => {
          let tooltip = params[0].axisValueLabel + '<br/>';
          params.forEach((param: any) => {
            if (param.data !== null) {
              tooltip += `${param.marker} ${param.seriesName}: ${param.data} 次提交<br/>`;
            }
          });
          return tooltip;
        }
      },
      legend: {
        data: topAuthors,
        top: '10%',
        type: 'scroll' // Enable scrolling for long lists
      },
      xAxis: {
        type: 'category',
        data: xAxisData,
        axisLabel: {
          interval: interval,
          rotate: 45
        }
      },
      yAxis: {
        type: 'value',
        name: '提交次数'
      },
      series: series,
      dataZoom: [
        {
          type: 'inside',
          start: 0,
          end: 100
        },
        {
          type: 'slider',
          start: 0,
          end: 100,
          bottom: 10
        }
      ]
    };
  };

  // Commit frequency distribution
  const getCommitFrequencyDistributionChartOption = () => {
    // Limit to last 30 days for better readability
    const recentData = statistics.commit_frequency_distribution.slice(0, 30);
    const data = recentData.map(d => [d.date, d.commit_count]);
    
    return {
      title: {
        text: '代码提交频率分布 (最近30天)',
        left: 'center'
      },
      tooltip: {
        trigger: 'axis',
        formatter: (params: any) => {
          const [date, count] = params[0].data;
          return `${date}<br/>提交次数: ${count} 次`;
        }
      },
      xAxis: {
        type: 'category',
        data: recentData.map(d => d.date),
        axisLabel: {
          rotate: 45
        }
      },
      yAxis: {
        type: 'value',
        name: '提交次数'
      },
      series: [{
        data: data,
        type: 'bar',
        itemStyle: {
          color: '#1f77b4'
        }
      }]
    };
  };

  // Commit size distribution
  const getCommitSizeDistributionChartOption = () => {
    const sizeLabels: { [key: string]: string } = {
      'small': '小提交 (≤10行)',
      'medium': '中等提交 (11-100行)',
      'large': '大提交 (101-500行)',
      'huge': '巨型提交 (>500行)'
    };

    const colors = ['#52c41a', '#1890ff', '#faad14', '#f5222d'];
    
    return {
      title: {
        text: '提交规模分布',
        left: 'center'
      },
      tooltip: {
        trigger: 'axis',
        formatter: (params: any) => {
          const item = params[0];
          const sizeData = statistics.commit_size_distribution[item.dataIndex];
          return `${sizeLabels[sizeData.size_range]}<br/>` +
                 `提交数量: ${sizeData.count} 次<br/>` +
                 `代码行数范围: ${sizeData.min_lines}-${sizeData.max_lines === 2147483647 ? '∞' : sizeData.max_lines} 行`;
        }
      },
      xAxis: {
        type: 'category',
        data: statistics.commit_size_distribution.map(d => sizeLabels[d.size_range] || d.size_range),
        axisLabel: {
          interval: 0,
          rotate: 0
        }
      },
      yAxis: {
        type: 'value',
        name: '提交次数'
      },
      series: [{
        data: statistics.commit_size_distribution.map((d, index) => ({
          value: d.count,
          itemStyle: {
            color: colors[index % colors.length]
          }
        })),
        type: 'bar',
        barWidth: '60%'
      }]
    };
  };

  // Programming efficiency trends
  const getEfficiencyTrendsChartOption = () => {
    // Limit to recent data for better readability
    const recentData = statistics.efficiency_trends.slice(-30);
    
    return {
      title: {
        text: '编程效率趋势 (最近30天)',
        subtext: '效率比例 = 新增代码 / (新增代码 + 删除代码)',
        left: 'center'
      },
      tooltip: {
        trigger: 'axis',
        formatter: (params: any) => {
          const data = recentData[params[0].dataIndex];
          return `${data.date}<br/>` +
                 `效率比例: ${(data.efficiency_ratio * 100).toFixed(1)}%<br/>` +
                 `总变更: ${data.total_changes} 行`;
        }
      },
      xAxis: {
        type: 'category',
        data: recentData.map(d => d.date),
        axisLabel: {
          rotate: 45
        }
      },
      yAxis: {
        type: 'value',
        name: '效率比例',
        min: 0,
        max: 1,
        axisLabel: {
          formatter: (value: number) => `${(value * 100).toFixed(0)}%`
        }
      },
      series: [{
        data: recentData.map(d => d.efficiency_ratio),
        type: 'line',
        smooth: true,
        lineStyle: {
          color: '#722ed1',
          width: 3
        },
        itemStyle: {
          color: '#722ed1'
        },
        areaStyle: {
          color: {
            type: 'linear',
            x: 0,
            y: 0,
            x2: 0,
            y2: 1,
            colorStops: [{
              offset: 0, color: 'rgba(114, 46, 209, 0.3)'
            }, {
              offset: 1, color: 'rgba(114, 46, 209, 0.1)'
            }]
          }
        }
      }],
      markLine: {
        data: [{
          yAxis: 0.5,
          label: {
            formatter: '平衡线 (50%)'
          },
          lineStyle: {
            color: '#999',
            type: 'dashed'
          }
        }]
      }
    };
  };

  // Hot files table component
  const HotFilesTable = () => {
    if (!statistics?.hot_files) {
      return (
        <div className="empty-state">
          <h3>暂无热点文件数据</h3>
        </div>
      );
    }

    const topFiles = statistics.hot_files.slice(0, 10);

    return (
      <div className="table-container">
        <h3 className="table-title">热点文件 TOP 10</h3>
        <table className="hot-files-table">
          <thead>
            <tr>
              <th>文件路径</th>
              <th>修改次数</th>
              <th>新增行数</th>
              <th>删除行数</th>
              <th>最后修改</th>
            </tr>
          </thead>
          <tbody>
            {topFiles.map((file, index) => (
              <tr key={index}>
                <td>
                  <span className="file-path-full" title={file.file_path}>
                    {file.file_path}
                  </span>
                </td>
                <td>{file.change_count}</td>
                <td>{file.total_additions}</td>
                <td>{file.total_deletions}</td>
                <td>{new Date(file.last_modified).toLocaleDateString()}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    );
  };

  // Commit message words (using word cloud)
  const getCommitMessageWordsChartOption = () => {
    const words = statistics.commit_message_words.slice(0, 30);
    
    return {
      title: {
        text: '提交消息高频词词云',
        left: 'center'
      },
      tooltip: {
        show: true,
        formatter: (params: any) => {
          return `${params.data.name}<br/>出现次数: ${params.data.value} 次`;
        }
      },
      series: [{
        type: 'wordCloud',
        sizeRange: [12, 60],
        rotationRange: [-90, 90],
        rotationStep: 45,
        gridSize: 8,
        shape: 'circle',
        width: '100%',
        height: '100%',
        textStyle: {
          fontFamily: 'sans-serif',
          fontWeight: 'bold',
          color: function () {
            // Random color
            return 'rgb(' + [
              Math.round(Math.random() * 160),
              Math.round(Math.random() * 160),
              Math.round(Math.random() * 160)
            ].join(',') + ')';
          }
        },
        emphasis: {
          textStyle: {
            shadowBlur: 10,
            shadowColor: '#333'
          }
        },
        data: words.map((w, index) => ({
          name: w.word,
          value: w.count,
          textStyle: {
            color: `rgb(${Math.round(100 + 155 * (index / words.length))}, ${Math.round(100 + 155 * (index / words.length))}, ${Math.round(200 - 100 * (index / words.length))})`
          }
        }))
      }]
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
      
      {/* Hourly Commit Distribution Heatmap */}
      <div className="chart-card">
        <ReactECharts
          option={getHourlyCommitDistributionChartOption()}
          style={{ height: '400px' }}
          notMerge={true}
        />
      </div>
      
      {/* Author Activity Trends */}
      <div className="chart-card">
        <ReactECharts
          option={getAuthorActivityTrendsChartOption()}
          style={{ height: '400px' }}
          notMerge={true}
        />
      </div>
      
      {/* Commit Frequency Distribution */}
      <div className="chart-card">
        <ReactECharts
          option={getCommitFrequencyDistributionChartOption()}
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
      
      {/* Commit Size Distribution */}
      <div className="chart-card">
        <ReactECharts
          option={getCommitSizeDistributionChartOption()}
          style={{ height: '300px' }}
          notMerge={true}
        />
      </div>
      
      {/* Programming Efficiency Trends */}
      <div className="chart-card">
        <ReactECharts
          option={getEfficiencyTrendsChartOption()}
          style={{ height: '400px' }}
          notMerge={true}
        />
      </div>
      
      {/* Commit Message Words */}
      <div className="chart-card">
        <ReactECharts
          option={getCommitMessageWordsChartOption()}
          style={{ height: '400px' }}
          notMerge={true}
        />
      </div>
      
      {/* Hot Files Table */}
      <div className="chart-card full-width">
        <HotFilesTable />
      </div>
    </div>
  );
};

export default StatisticsCharts;