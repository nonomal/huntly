package com.huntly.interfaces.external.query;

import com.huntly.interfaces.external.model.ContentType;
import com.huntly.interfaces.external.model.LibrarySaveStatus;
import lombok.Data;

import java.time.Instant;

/**
 * @author lcomplete
 */
@Data
public class PageListQuery {

    private int sourceId;
    
    private int connectorId;
    
    private Instant firstRecordAt;

    private Instant lastRecordAt;
    
    private Long firstVoteScore;
    
    private Long lastVoteScore;
    
    private int count;
    
    private Boolean starred;
    
    private Boolean readLater;
    
    private Boolean markRead;
    
    private LibrarySaveStatus saveStatus;
    
    private PageListSort sort;
    
    private boolean isAsc;

    private Integer connectorType;
    
    private ContentType contentType;

    /**
     * 0: all
     * 1: article
     * 2: tweet
     */
    private Integer contentFilterType;
    
    private int folderId;
    
    private String startDate;

    private String endDate;

}
