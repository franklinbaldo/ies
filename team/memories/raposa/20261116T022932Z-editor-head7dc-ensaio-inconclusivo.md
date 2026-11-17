---
type: AgentMemory
title: "🦊 #851: ablação comprovada, replay completo ainda pendente"
description: "No HEAD 7dc882be, Texugo verificou ablação antes/depois; tentativa isolada da Raposa não produziu replay executável final."
agent: raposa
timestamp: "2026-11-16T02:29:32Z"
tags: [ies, raposa, editor-nativo, cassette, ablation, not-run]
---

# Retomar em #851

**Fatos conferidos:** PR [#852](https://github.com/franklinbaldo/ies/pull/852) segue Draft no HEAD `7dc882be9c0f41dc4ed0d7e1f190679656928288`, inicialmente `behind=0`. O CI rápido [#43786521426](https://github.com/franklinbaldo/ies/actions/runs/43786521426) executou checkout, Rustfmt e Prek e terminou SUCCESS; isso **não** executou replay oficial. A revisão de [🦡 Texugo na PR](https://github.com/franklinbaldo/ies/pull/852) reproduziu, em cópias isoladas, compilação e seis `editor_montar` originais UnB/GO antes/depois da ablação, `1 PASS` em cada; quatro testes de `documento_salvar_cli` comuns preservados. Não demonstra CLI salvar, anti-IES ou efeitos de escrita.

**Tentativa própria:** o Ifro respondeu, e uma cópia Git isolada da PR foi conferida exatamente no SHA `7dc882be` e tree `61e64484cc5f2907a51e71e5d53623aa4d885214`; cache tinha ZIPs originais UnB/GO. Tentativa de `cargo test --locked --offline -p ies --bin ies replay_editor_nativo_no_acervo -- --ignored --nocapture` encontrou dependências de runtime (PATH/toolchain/OpenSSL) e, após instalar dependências temporárias no contêiner, ficou sem resultado conclusivo na janela de observação; chamadas de controle do Ifro expiraram. **Teste próprio no SHA final: NOT_VERIFIED**, não PASS. Evitar iniciar cópias concorrentes; inspecionar e encerrar processos antes de novo ensaio. Nenhuma chamada ao SEI (0 GET / 0 POST a instalações), nenhuma escrita de produção.

**Pendente:** replay de CLI real e anti-IES com comparação bilateral independente às respostas originais e SQLite quando aplicável; negativos e CI oficial com steps no HEAD de merge: **NOT_RUN**. Live: **NOT_RUN**. Manter `#852` em Draft, sem segundo parser, sem BDD, sem liberação de escrita remota. Conservar a recusa existente de origem e ausência de retry; coordenar a revisão 🦅 antes de Ready.
