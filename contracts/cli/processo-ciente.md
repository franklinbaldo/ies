---
type: CommandContract
title: Ciência no processo pela interface Web
operation: processo.ciente
command: ies processo ciente NUP [--confirmar]
effect: write-process-state
implementation: partial
---

# `ies processo ciente NUP [--confirmar]` — #271

Prévia sem escrita e confirmação pelo próprio botão `cienciaProcesso()` da
barra do processo; o GET da ação `procedimento_ciencia` **é escrita**.
O comando aceita `--unidade` e `--json` globais, valida o NUP antes
de ler arquivos privados, usa a sessão Web atual e não segue 302.

## Sequência observável

1. Abrir `ies processo NUP` com unidade exigida, sem andamentos,
   propriedades nem marcadores. Examinar a ação de ciência uma vez na
   barra da árvore; link da tela do nó `arvore_visualizar` validado.
2. Prévia sem `--confirmar`: apenas o custo da abertura, nenhum GET
   de `procedimento_ciencia` e nenhuma escrita. Publicar a unidade.
3. Com `--confirmar`: GET da tela do nó, conferir a URL assinada de
   `cienciaProcesso()` associada ao processo e emitir **um único GET**
   de `procedimento_ciencia`. Nunca retry, inclusive após timeout.
4. Extrair do aceite a `id_atividade` de literal `atualizarArvore`
   associado ao `id_procedimento` e ao sinal
   `procedimento_visualizar_ciencias=1`. URL assinada e hash não saem
   na resposta, nos logs nem no `Debug`.
5. Abrir novamente a árvore e exigir a ação da Lista de Ciências
   `NosAcoes` do próprio processo, com `acao=protocolo_ciencia_listar`.
   Um GET da lista, conferir a atividade no checkbox
   `chkInfraItem` (ou `chkInfraItem1` no SEI 5.0).
   Duplicidade de atividades ou lista ilegível não é sucesso.

Acesso sem botão, tela inválida e cookie recusado falham com saída 1
**antes** de enviar a escrita. Depois do GET mutador, nenhuma falha
pode provocar nova escrita: a árvore é relida, e ausência de prova
sai 3 `efeito_incerto`, nunca sucesso presumido. Antes de tentar
outra vez, confira a Lista de Ciências. Uma ciência por usuário por dia.

## Subcódigos observáveis ou de recusa

| Subcódigo | Condição |
| --- | --- |
| `ciencia_nao_oferecida_na_barra` | Barra sem ação de ciência |
| `unidade_nao_publicada_antes_da_ciencia` | Unidade ausente na abertura |
| `barra_de_ciencia_fora_da_forma` | Atributos da barra recusados |
| `ciencia_repetida_na_barra` | Mais de uma ação |
| `tela_de_ciencia_nao_publicada` | Tela do nó ausente |
| `funcao_de_ciencia_ausente_na_tela` | Função ausente |
| `funcao_de_ciencia_fora_da_forma` | Função inválida |
| `funcoes_de_ciencia_repetidas` | Função duplicada |
| `endereco_da_ciencia_nao_reconhecido` | Endereço do GET inválido |
| `lista_de_ciencias_repetida_na_arvore` | Mais de uma lista |
| `lista_de_ciencias_fora_da_forma` | Ação inválida |
| `endereco_da_lista_de_ciencias_nao_reconhecido` | Link inválido |
| `tela_de_ciencia_redirecionada` | 302 antes da escrita |
| `aceite_da_ciencia_sem_atividade` | Aceite sem atividade |
| `atividades_da_ciencia_ambiguas` | Aceite com múltiplas atividades |
| `lista_de_ciencias_fora_da_forma_observada` | HTML da lista não reconhecido |
| `atividades_repetidas_na_lista_de_ciencias` | Checkbox duplicado |
| `efeito_incerto_na_ciencia` | Escrita sem verificação conclusiva |
| `validacao_da_infra_na_ciencia_do_processo` | GET mutador devolveu validação Infra 200; preservar mensagem e ler novamente |
| `atividade_do_aceite_do_processo_nao_reconhecida` | Resposta do GET mutador sem identidade de atividade comprovável |



- `ciencia_nao_oferecida_na_barra`
- `unidade_nao_publicada_antes_da_ciencia`
- `barra_de_ciencia_fora_da_forma`
- `ciencia_repetida_na_barra`
- `tela_de_ciencia_nao_publicada`
- `funcao_de_ciencia_ausente_na_tela`
- `funcao_de_ciencia_fora_da_forma`
- `funcoes_de_ciencia_repetidas`
- `endereco_da_ciencia_nao_reconhecido`
- `lista_de_ciencias_repetida_na_arvore`
- `lista_de_ciencias_fora_da_forma`
- `endereco_da_lista_de_ciencias_nao_reconhecido`
- `tela_de_ciencia_redirecionada`
- `aceite_da_ciencia_sem_atividade`
- `atividades_da_ciencia_ambiguas`
- `lista_de_ciencias_fora_da_forma_observada`
- `atividades_repetidas_na_lista_de_ciencias`
- `efeito_incerto_na_ciencia`
- `validacao_da_infra_na_ciencia_do_processo`
- `atividade_do_aceite_do_processo_nao_reconhecida`

Falhas de transporte são tipadas pela infraestrutura. `--json`
retorna `numero`, `id_procedimento`, `unidade`,
`confirmado`, `verificado`, `id_atividade` (`null` se não
observada), `requisicoes`, e origem `web`.
Nenhum identificador de atividade é inferido de andamento genérico.

## Orçamento de requisições

| Caminho | Custo acima da abertura |
| --- | --- |
| NUP inválido | zero no total |
| Prévia | zero |
| Tela do nó recusada | 1 GET |
| Confirmação, atividade observada | 1 GET tela + 1 GET escrita + abertura da árvore nova + 1 GET lista |
| Confirmação, aceite sem atividade | 1 GET tela + 1 GET escrita + abertura da árvore nova; sem consulta inútil da lista |

O custo da abertura depende do índice: 2 GET por link de trabalho
guardado ou 3 GET a frio pelo Controle; a verificação frequentemente
custa 1 GET pelo link da árvore. Um processo com pastas fechadas pode
exigir 1 GET adicional. Os testes integrados sintéticos de orçamento verificam 3 GETs na prévia,
7 GETs com ciência seguida de prova e 6 GETs quando o aceite não
fornece atividade, sempre um único GET mutador e sem retry. Não afirmar live.

## Evidências e pendências

Oráculo: Todos `src/todos/ciclo.py` @323eb01 e testes de ciência;
metadados das cassettes em `contracts/evidencias/processo-ciente.md`,
sem replay local completo. [Decisão](../../docs/decisoes/processo-ciente.md)
e [relatório](../../docs/validacao-live/2026-11-05-processo-ciente.md).
Os testes com transporte injetado afirmam também a preservação de causa
no GET mutador (403, validação Infra 200 e falha de rede); no efeito
incerto, causa tipada, status HTTP e mensagem saneada são publicados
sem repetir a escrita. Execução live em treinamento, inclusive recusa
de segunda ciência, e replay específico permanecem **NOT_RUN**.

A `id_atividade` só é aceita se a chamada literal
`atualizarArvore` constar em código de script ativo da resposta.
Texto somente em comentário, string, script aninhado em
`<template>`, ramo condicional (`if(false)`) ou função arbitrária não
invocada, ou instrução inalcançável após `return`/`throw` não é prova;
essas formas geram aceite não reconhecido
e a verificação posterior não presume sucesso. Regressões sintéticas
RED/GREEN documentadas na [decisão](../../docs/decisoes/processo-ciente.md).
