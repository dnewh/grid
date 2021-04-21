/**
 * Copyright 2018-2021 Cargill Incorporated
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
import React, {useEffect} from 'react'
import PropTypes from 'prop-types';
import Icon from '@material-ui/core/Icon';
import { useTable, useFilters, usePagination } from 'react-table';

import './Table.scss';
import _ from 'lodash';

export function Table({ columns, data, filterTypes, filters }) {
  const {
    getTableProps,
    getTableBodyProps,
    headerGroups,
    prepareRow,
    page,
    canPreviousPage,
    canNextPage,
    nextPage,
    previousPage,
    setFilter,
    state: { pageIndex, pageSize },
  } = useTable(
    {
      columns,
      data,
      initialState: { pageIndex: 0 },
      filterTypes,
    },
    useFilters,
    usePagination,
  )

  useEffect(() => {
    filters.map(({type, value}) => {
      setFilter(type, value);
      return null;
    })
  }, [filters])

  const downloadXML = row => {
    let xml = _.find(row.original.product.properties, {'name': 'xml_data'})
    if (xml) {
      xml = xml.string_value;
      const filename = `${row.values.gtin}.xml`;
      const blob = new Blob([xml], { type: 'text/plain' });
      const el = document.createElement('a');
      el.setAttribute('href', window.URL.createObjectURL(blob));
      el.setAttribute('download', filename);
      el.dataset.downloadurl = ['text/plain', el.download, el.href].join(':');

      el.click();
    }
  }

  const Pagination = () => (
    <div className="pagination">
      <span className="results">{data.length} Results</span>
      <div className="rpp">
        <span className="label">Rows per page: </span>
        <span className="value">{pageSize}</span>
      </div>
      <span className="range">{`${pageIndex + 1}-${Math.min(data.length, (pageIndex +1) * 10)} of ${data.length}`}</span>
      <Icon className="page-step" onClick={() => previousPage()} disabled={!canPreviousPage}>
        chevron_left
      </Icon>
      <Icon className="page-step" onClick={() => nextPage()} disabled={!canNextPage}>
        chevron_right
      </Icon>
    </div>
  );

  return (
    <>
      <Pagination />
      <div className="table-wrapper">
        <table {...getTableProps()}>
          <thead>
            {headerGroups.map(headerGroup => (
              <tr {...headerGroup.getHeaderGroupProps()}>
                {headerGroup.headers.map(column => (
                  <th {...column.getHeaderProps()}>{column.render('Header')}</th>
                ))}
                <th>Action</th>
              </tr>
            ))}
          </thead>
          <tbody {...getTableBodyProps()}>
            {page.map(row => {
              prepareRow(row)
              return (
                <tr {...row.getRowProps()}>
                  {row.cells.map(cell => {
                    return <td {...cell.getCellProps()}>{cell.render('Cell')}</td>
                  })}
                  <td>
                    <div className="action-button-wrapper">
                      <div className="action-button">
                        <Icon>more_horiz_icon</Icon>
                        <span className="action-options">
                          <button
                            className="key-action-btn"
                            title="Download XML"
                            type="button"
                            onClick={e => {
                              e.preventDefault();
                              downloadXML(row);
                            }}
                          >
                            <Icon>code</Icon>
                          </button>
                        </span>
                      </div>
                    </div>
                  </td>
                </tr>
              )
            })}
          </tbody>
        </table>
      </div>
      <Pagination />
    </>
  )
}

Table.propTypes = {
  columns: PropTypes.array.isRequired,
  data: PropTypes.array.isRequired,
  filterTypes: PropTypes.object,
  filters: PropTypes.array,
}

Table.defaultProps = {
  filterTypes: {},
  filters: []
}
