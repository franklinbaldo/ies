---
type: AgentMemory
title: "BDD-633: recusa tipada e correção do limite de linhas do teste"
description: "A PR #688 foi reconciliada com main por merge real; extraí o preparo do Controle após Clippy too_many_lines. GREEN oficial e confronto semântico ainda dependem de execução."
agent: castor
timestamp: "2026-11-10T17:32:23Z"
tags: [ies, anti-ies, bdd, rust, clippy, validacao-acesso, sqlite, cassettes]
---

# Próximo encaixe

**Fato:** a [PR #688](https://github.com/franklinbaldo/ies/pull/688), da [issue #633](https://github.com/franklinbaldo/ies/issues/633), preserva `ProcessoExigeValidacaoDeAcesso` quando a tela do link `procedimento_trabalhar` guardado requer validação, em vez de tratar a recusa como link vencido. O merge real `6e89113` incorporou a `main` sem sobreposição de arquivos; no [CI #42864175511](https://github.com/franklinbaldo/ies/actions/runs/42864175511), a suíte Rust passou, mas Clippy recusou `estado()` no teste por **111/100 linhas**. Extraí `guardar_controle_parcial` nos commits `5bbee2f` e `0186dc7`, sem desativar lint. O [CI rápido #42865615813](https://github.com/franklinbaldo/ies/actions/runs/42865615813) passou; o CI oficial desse novo head foi iniciado, não conferido ao escrever.

**Provas distintas:** a CLI Rust real ↔ TLS local ↔ anti-IES ↔ SQLite v36 teve suíte PASS no SHA anterior, com dois GET e zero POST; no SHA novo, aguardava o CI oficial. A inspeção dos três cards GO (48 interações) confirmou estrutura e hashes, mas não a ação da modal. O HTML da exigência é **hipótese sintética**: confrontos independentes IES↔cassette e anti-IES↔cassette `NOT_RUN`; treinamento/live `NOT_RUN`. Não chamar BDD-633-02 concluído: falta explicar especificamente o arquivo de credenciais ausente; BDD-633-01/03 seguem pendentes.

**Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388) depois da correção. A distinção entre silêncio observado e não observado reaparece aqui como **link envelhecido** versus **autorização explicitamente exigida**; nunca transformar a segunda em busca alternativa.

**CONTINUAR:** validar sete gates no head reconciliado, re-sincronizar a branch após esta memória direta na main e pedir revisão 🦅. Não executar POST no SEI externo sem autorização, não criar subcódigo e não incluir senhas em fixtures ou logs.
