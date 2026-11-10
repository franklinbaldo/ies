---
type: ImplementationDecision
title: Anotação de processo observada antes da escrita
operation: processo.anotar
implementation: partial
status: adopted-for-scope
scope: somente prévia da anotação por GET, sem POST de escrita
selected_strategy: web-formulario-servido-com-identidade
alternatives: web-formulario-servido-com-identidade,estado-inferido-da-barra,url-assinada-construida
evidence_level: todos-unb-e-cassettes-publicas
live_validation: not-run
---

# Prévia de anotação do processo (#638)

## Decisão

O Todos observou a tela `frmAnotacaoCadastro`, com `hdnIdProtocolo` e `txaDescricao`. A caixa pode estar vazia; não inferir anotação inexistente de link ausente. A primeira fatia consulta o link assinado **publicado na barra** do processo aberto na unidade da sessão, valida o formulário e informa o texto anterior e o pretendido. Não grava nem oferece confirmação. É a opção segura contra a alternativa de fabricar uma URL ou inferir o estado da mensagem/ícone.

Reaproveitar o leitor empregado por `processo enviar --remover-anotacoes` e servir a mesma forma no anti-IES a partir do SQLite. O cenário sintético contém `anotacao_origem: Some("")`, que representa observação explícita de vazio, diferente de `None` (não observado). A prévia falha antes de escrever se a unidade ou formulário não for verificado.

## Provas e limites

[Contrato](../../contracts/cli/processo-anotar.md), [BDD na issue #638](https://github.com/franklinbaldo/ies/issues/638), [cassette UnB](../../cassettes/unb/processo-escritas-unb--47c0acc994ac.md), [cassette Goiás](../../cassettes/go/processo-escritas-go--841b0341a184.md).

A verificação estrutural dos ZIPs pela Action não substitui replay semântico. CLI real → anti-IES → SQLite só comprova integração sintética e ausência de escrita no caminho ensaiado. Confirmar, remover e editar anotação preexistente permanecem fora desta fatia. Produção e treinamento: `NOT_RUN`.
