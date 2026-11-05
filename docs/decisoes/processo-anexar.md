---
type: ImplementationDecision
title: Anexação de processo com prévia Web e confirmação fechada
operation: processo.anexar
implementation: partial
status: adopted-for-scope
scope: prévia somente, duas consultas AJAX de leitura e recusa da escrita até teste integrado de verificação
selected_strategy: web-formulario-da-tela-com-duas-consultas-observadas
alternatives: web-formulario-da-tela-com-duas-consultas-observadas,web-sem-verificacao-de-assinaturas,rest-wssei
evidence_level: todos-anexacao-unb-409-20261015-e-testes-sinteticos
live_validation: not-run
---

# Decisão: iniciar pela prévia de anexação (issue #270)

## Evidência e risco

O oráculo `franklinbaldo/todos/src/todos/anexacao.py` relata observação UnB 4.0.9 em 15/10/2026: `procedimento_anexar`, `frmProcedimentoAnexar`, campos `txtProtocolo`, `txtIdentificacaoProtocolo`, `hdnIdProtocolo`, `hdnAnexacao`, botão `sbmAnexar`, consulta `protocolo_RI1023` e verificação `procedimento_anexacao_verificar`. O código de referência identifica sucesso por 302 e releitura de `Lista de Processos Anexados`. A operação não tem desfazimento comum, e anexar processo com documento não assinado pode impedir assinatura.

## Alternativas

1. **Tela servida e duas consultas read-only** (escolhida): confere processo e controles, rejeita destino/assinaturas ambíguos; prévia concreta sem escrita e sem inventar URLs assinadas.
2. **Pular consulta de assinaturas**: reduz uma requisição, mas omite aviso relevante para uma operação potencialmente irreversível.
3. **REST/WSSEI**: não é a interface priorizada pelo IES.

## Decisão e limite

Uma peça por responsabilidade: leitor/decisor puro em `src/processo_anexar.rs`, ação da barra em `src/arvore_processo/anexar.rs`, transporte e contagem compartilhados em `processo_concluir::escrita`; nenhuma modificação em `transporte` ou `formulario_web`. A CLI só libera prévia. O `--confirmar` recusa antes de rede enquanto não houver teste integrado de envio único, conciliação 302 e verificação por releitura; não enfraquecer gates para habilitar escrita.

## Revisão

Completar a evidência pelo replay de cassettes; confirmar forma exata das duas chamadas AJAX; só depois habilitar confirmação em treinamento com prévia → um POST → uma releitura e registro do resíduo. Live NOT_RUN, zero requisições.
