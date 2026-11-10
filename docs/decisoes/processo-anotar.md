---
type: ImplementationDecision
title: Anotação de processo observada antes e depois de uma escrita
operation: processo.anotar
implementation: partial
status: adopted-for-scope
scope: consulta sem escrita, prévia e confirmação única com releitura
selected_strategy: web-formulario-servido-com-identidade
alternatives: web-formulario-servido-com-identidade,estado-inferido-da-barra,url-assinada-construida
evidence_level: todos-unb-e-cassettes-publicas
live_validation: not-run
---

# Anotação de processo (#638)

## Alternativas consideradas

1. **web-formulario-servido-com-identidade (escolhida):** seguir o link assinado presente na barra do processo aberto, conferir o ID e ler a anotação do formulário `frmAnotacaoCadastro`.
2. **estado-inferido-da-barra:** deduzir a anotação pelo ícone ou pela presença de link, sem consultar o formulário. Não distingue ausência observada de ausência de observação.
3. **url-assinada-construida:** fabricar a URL a partir do NUP. Não comprova oferta da ação nem preserva a assinatura servida.

## Comparação

| Critério | Formulário servido | Barra inferida | URL construída |
| --- | --- | --- | --- |
| Identidade por `hdnIdProtocolo` | sim | não | não |
| Texto atual observado | sim | não | não |
| Respeita o link realmente oferecido | sim | parcial | não |
| Escrita na prévia | nenhuma | nenhuma | nenhuma |

## Decisão

O Todos observou `frmAnotacaoCadastro`, `hdnIdProtocolo` e `txaDescricao`. A fatia BDD-ANOT-01 consulta o link assinado **publicado na barra** de um processo aberto na unidade da sessão, valida a identidade do formulário e apresenta o conteúdo anterior e o texto pedido. Não grava.

Reutilizar o leitor usado por `processo enviar --remover-anotacoes` e projetar o formulário do anti-IES a partir do estado SQLite. `anotacao_origem: Some("")` significa vazio explicitamente observado; `None` significa não observado. Só publicar o link da anotação na unidade de origem aberta. Preservar a ação independente de anexar processo que já existe na `main`. Não executar POST na prévia.

## Evidências verificáveis

- [Contrato da CLI](../../contracts/cli/processo-anotar.md) e [matriz de evidências](../../contracts/evidencias/processo-anotar.md).
- [BDD-ANOT-01 na issue #638](https://github.com/franklinbaldo/ies/issues/638).
- [Teste Rust vertical CLI ↔ anti-IES ↔ SQLite](../../tests/aies_anotacao_cli.rs).
- [Cassette UnB](../../cassettes/unb/processo-escritas-unb--47c0acc994ac.md) e [cassette Goiás](../../cassettes/go/processo-escritas-go--841b0341a184.md). Integridade de ZIPs verificada anteriormente em [Actions #42772050666](https://github.com/franklinbaldo/ies/actions/runs/42772050666), não confrontação semântica.
- [CI de diagnóstico](https://github.com/franklinbaldo/ies/actions/runs/42772884266): 2165 testes passaram; um teste detectou link duplicado; o gate `check_design` apontou as seções e os vínculos ausentes. Esta decisão inclui os vínculos exigidos e a correção do link está na mesma PR.

## Extensão BDD-ANOT-02: decisão causal

A confirmação reutiliza a mesma derivação da tela já observada, em vez de
fabricar um formulário no cliente ou repetir a escrita ao receber resposta
ambígua. A transição do anti-IES modifica o campo textual existente no SQLite;
na segunda leitura HTTP é retornado o mesmo texto. A origem fica restrita às
instalações de treinamento, antes de qualquer requisição de `--confirmar`.

Comparadas as alternativas, aceitar apenas o texto na releitura permitiria
atribuir à CLI uma alteração concorrente; aceitar apenas o 302 não provaria
persistência. A estratégia exige **ambas**: 302 do POST com identidade/ação
conferidas e GET independente do formulário contendo o texto solicitado,
sem repetir a escrita. Recusa, perda de resposta ou releitura inconclusiva
após tentativa de POST resultam em `efeito_incerto` (saída 3).
Anotação não vazia preexistente é recusada antes do POST, sem ampliar o
BDD para sobrescrita. [PR #654](https://github.com/franklinbaldo/ies/pull/654)
contém os testes da CLI real.

Os dois cards originais UnB/GO foram inspecionados outra vez no
[Actions #42786619821](https://github.com/franklinbaldo/ies/actions/runs/42786619821):
SHA e estrutura das 77 interações **PASS**, mas isso não compara corpos
nem efeitos dos POSTs. Paridade independente IES/anti-IES com cassette e
treinamento continuam `NOT_RUN`.

## Extensão BDD-ANOT-03: consulta sem alteração

Sem `--texto`, a CLI retorna o conteúdo do mesmo formulário servido e
conferido por identidade, sem executar POST. A projeção anti-IES já deriva
`txaDescricao` de `anotacao_origem` no SQLite: `Some("")` é ausência
observada, enquanto `None` proíbe inferir ausência e nem oferece o
formulário. A alternativa de deduzir o texto pela barra permanece rejeitada.
As duas regressões da CLI real/anti-IES/SQLite distinguem texto, vazio
explícito e imutabilidade da consulta; não constituem prova de paridade
com as respostas históricas nem validação externa.

## Limitações e revisão

A integração da CLI com o anti-IES/SQLite é sintética. Os ZIPs não provam que a mesma resposta e a projeção do anti-IES coincidem semanticamente, nem provam efeito de POST. Confronto bilateral com cassettes: **NOT_RUN**; treinamento e produção: **NOT_RUN**, sem requisições ou escrita externa. BDD-ANOT-04/05 (retirada e envio) ficam para próximas fatias. Rever o contrato após observação independente do formulário, sem elevar hipótese à condição de fato.
