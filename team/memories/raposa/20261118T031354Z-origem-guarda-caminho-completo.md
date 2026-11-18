---
type: AgentMemory
title: "🦊 Guarda antes da rede: auditoria do caminho completo de documento salvar"
description: "Revisão estática do pré-flight de confirmação: prévia/sessão locais, guarda antes de transporte; Cargo no HEAD ainda não executado."
agent: raposa
timestamp: "2026-11-18T03:13:54Z"
tags:
  - ies
  - documento-salvar
  - guarda-de-origem
  - evidencia
---

# Revisão curta da PR #857

Na [PR #857](https://github.com/franklinbaldo/ies/pull/857#issuecomment-6967896467), HEAD `5564d814`, conferi a sequência `tentar_salvar_documento` → `previa_local` → `com_sessao` → `executar`: a prévia e a leitura da sessão/índice são locais, e `executar` confere a origem permitida antes de criar o transporte `EnvioHttps`. É conclusão **estática** sobre esse caminho, não execução de rede ou prova universal de ausência de sockets.

`copia_documento::par` já recusa `u32::MAX` ao chamar `VersaoDocumento::seguinte()` na prévia. A regressão nova em `remoto::ler_par` prova, se executada, somente a robustez interna do mesmo limite; não é teste de CLI. Evitar atribuir a ela cobertura E2E.

SSH ao Ifro expirou novamente. No HEAD `5564d814`, o CI rápido de Rustfmt/Prek passou; Cargo, replay e live seguem **NOT_RUN**. DRAFT/HOLD sem merge; próximo passo útil é Cargo no SHA exato, depois CI/replays e confronto com os GETs originais SEI4/5. Não houve acesso ao SEI, nem autorização nova de produção.

Visitei o [Cafezinho IES #388](https://github.com/franklinbaldo/ies/issues/388); segue sem obrigação de mensagem. A direção vigente é [Cafezinho da equipe #21](https://github.com/franklinbaldo/cafezinho/issues/21) R48: Raposa permanece IES #851 e a Ficha #258 aguarda a fila P0.