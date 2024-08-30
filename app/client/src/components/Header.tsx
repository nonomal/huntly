import EnergySavingsLeafIcon from "@mui/icons-material/EnergySavingsLeaf";
import SearchBox from "./SearchBox";
import React, {useCallback, useEffect, useState} from "react";
import {WindowStateListenerType} from "../domain/electronTypes";
import {IconButton} from "@mui/material";
import SettingsOutlinedIcon from '@mui/icons-material/SettingsOutlined';
import DehazeIcon from '@mui/icons-material/Dehaze';
import SettingModal from "./SettingModal";

export default function Header() {
  const [winMaximized, setWinMaximized] = useState(false);
  const [fullScreen, setFullScreen] = useState(false);
  const [focused, setFocused] = useState(false);
  const [settingModalOpen, setSettingModalOpen] = useState(false);
  const inElectron = !!window.electron;
  const isMac = inElectron && window.electron.utilsBridge.isMac;

  useEffect(() => {
    if (!inElectron) {
      return;
    }
    const {utilsBridge} = window.electron;
    // setIsMac(utilsBridge.isMac);
    utilsBridge.addWindowStateListener(
      (type: WindowStateListenerType, state: boolean) => {
        switch (type) {
          case WindowStateListenerType.Maximized:
            setWinMaximized(state);
            break;
          case WindowStateListenerType.Fullscreen:
            setFullScreen(state);
            break;
          case WindowStateListenerType.Focused:
            setFocused(state);
            break;
        }
      }
    );
  }, []);

  const minimize = () => {
    window.electron.utilsBridge.minimizeWindow();
  };
  const maximize = () => {
    window.electron.utilsBridge.maximizeWindow();
    setWinMaximized(!winMaximized);
  };
  const close = () => {
    window.electron.utilsBridge.closeWindow();
  };

  const openSettingModal = useCallback(() => {
    setSettingModalOpen(true);
  }, []);

  const closeSettingModal = useCallback(() => {
    setSettingModalOpen(false);
  }, []);

  function toggleSidebar() {
    const rootEl = document.getElementById('root');
    rootEl?.classList.toggle('toggle-sidebar');
  }

  return <header className="title_bar">
    {!isMac && (
      <span className="logo-area pl-4 flex items-center text-sky-600 absolute font-bold">
        <span className={'slide-button'}>
          <IconButton onClick={toggleSidebar}>
            <DehazeIcon/>
          </IconButton>
        </span>
        <EnergySavingsLeafIcon className="h-6 w-6 mr-1"/>
        <span className={'logo-text'}>
          Huntly
        </span>
      </span>
    )}

    <SearchBox/>

    {
      (!inElectron || isMac) && <IconButton className={'setting-area mr-2 absolute right-0'} onClick={openSettingModal}>
        <SettingsOutlinedIcon className={'text-sky-600'}/>
      </IconButton>
    }

    <SettingModal open={settingModalOpen} onClose={closeSettingModal}></SettingModal>

    {!isMac && inElectron && (
      <div className="absolute right-0">
        <IconButton className={'mr-2 nodrag'} onClick={openSettingModal}>
          <SettingsOutlinedIcon className={'text-sky-600'}/>
        </IconButton>
        <button className="btn-size-ctrl" onClick={minimize}>
          {/* <HorizontalRuleIcon fontSize="small" /> */}
          <svg viewBox="0 0 11 11">
            <path d="M11,4.9v1.1H0V4.399h11z"/>
          </svg>
        </button>
        <button className="btn-size-ctrl" onClick={maximize}>
          {winMaximized ? (
            <svg viewBox="0 0 11 11">
              <path
                d="M7.9,2.2h-7C0.4,2.2,0,2.6,0,3.1v7C0,10.6,0.4,11,0.9,11h7c0.5,0,0.9-0.4,0.9-0.9v-7C8.8,2.6,8.4,2.2,7.9,2.2z M7.7,9.6 c0,0.2-0.1,0.3-0.3,0.3h-6c-0.2,0-0.3-0.1-0.3-0.3v-6c0-0.2,0.1-0.3,0.3-0.3h6c0.2,0,0.3,0.1,0.3,0.3V9.6z"/>
              <path d="M10,0H3.5v1.1h6.1c0.2,0,0.3,0.1,0.3,0.3v6.1H11V1C11,0.4,10.6,0,10,0z"/>
            </svg>
          ) : (
            <svg viewBox="0 0 11 11">
              <path
                d="M0,1.7v7.6C0,10.2,0.8,11,1.7,11h7.6c0.9,0,1.7-0.8,1.7-1.7V1.7C11,0.8,10.2,0,9.3,0H1.7C0.8,0,0,0.8,0,1.7z M8.8,9.9H2.2c-0.6,0-1.1-0.5-1.1-1.1V2.2c0-0.6,0.5-1.1,1.1-1.1h6.7c0.6,0,1.1,0.5,1.1,1.1v6.7C9.9,9.4,9.4,9.9,8.8,9.9z"/>
            </svg>
          )}
        </button>
        <button className="btn-size-ctrl btn-size-close" onClick={close}>
          <svg viewBox="0 0 11 11">
            <path
              d="M6.279 5.5L11 10.221l-.779.779L5.5 6.279.779 11 0 10.221 4.721 5.5 0 .779.779 0 5.5 4.721 10.221 0 11 .779 6.279 5.5z"/>
          </svg>
        </button>
      </div>
    )}
  </header>
}