---
type: AgentMemory
title: "🦊 #851: ensaio offline no HEAD exato sem replay concluído"
description: "No HEAD 0a7df75, clone limpo e ZIPs em cache; compilação Rust foi interrompida por pressão de recursos, sem PASS do replay."
agent: raposa
timestamp: "2026-11-16T03:31:41Z"
tags: [ies, documento-salvar, cassette, ifro, ci, not-run]
---

# Continuidade IES #851

**Fato:** [PR #852](https://github.com/franklinbaldo/ies/pull/852) segue Draft. Comparei `main@91cf0cfb` com `raposa/851-editor-nativo-20261115@0a7df75d`: `behind=0`, quatro arquivos de escopo. O checkout isolado no Ifro conferiu `HEAD=0a7df75dbae9c3aab64e1b3eb302a56fc9b0cd0a` e `git status` limpo; cache offline incluía ZIPs UnB/GO. Nenhuma branch/PR nova.

**Tentativa controlada:** `cargo fetch --locked` em imagem Rust 1.91.1 no Ifro obteve dependências; `cargo test --locked --offline --bin ies replay_editor_nativo_no_acervo -- --ignored` iniciou compilação sem acesso do contêiner à rede. A compilação/teste não concluiu audivelmente: limite de tempo do controlador, disputa por lock de `target` entre tentativas e pressão de RAM na máquina de 4 GB. As duas execuções de contêiner iniciadas por esta rodada receberam solicitação de parada, e os processos próprios receberam TERM; nenhum `test result` nem código de sucesso foi obtido. **Replay no HEAD: NOT_VERIFIED**, não PASS. Evitar repetir duas compilações simultâneas e usar imagem/target configurados para baixa memória.

**Provas separadas:** teste opt-in de triagem de seis GETs UnB/GO publicado anteriormente pelo Texugo em SHA antigo não valida CLI completa; IES↔cassette e anti-IES↔cassette independentes: **NOT_RUN**; CLI↔anti-IES↔SQLite de salvar: **NOT_RUN**; CI oficial no HEAD: **NOT_RUN**; treinamento live: **NOT_RUN**, nenhum GET/POST SEI. [CI rápido #43792626796](https://github.com/franklinbaldo/ies/actions/runs/43792626796) = SUCCESS no SHA, apenas lint/pre-commit, sem replay.

**Decisão:** nenhum patch de produção, classificador ou erro novo; preservar guarda `sei.test`, recusa, 1 POST e releitura independente. Manter Draft até replay e revisão 🦅. [Handoff #851](https://github.com/franklinbaldo/ies/issues/851); [recado #19](https://github.com/franklinbaldo/raposa/issues/19). Sem BDD, produção ou escrita SEI real.
