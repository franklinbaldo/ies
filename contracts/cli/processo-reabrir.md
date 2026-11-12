---
type: CommandContract
title: Reabrir o processo na unidade da sessão
operation: processo.reabrir
command: ies processo reabrir NUP [--confirmar]
effect: write-process-state
implementation: partial
---

# `ies processo reabrir NUP [--confirmar]` (Refs #172)

Reabre o processo **na unidade da sessão**, uma vez e por pedido
explícito, pela ação "Reabrir Processo" que a barra do nó `PROCESSO` da
árvore publica. É o espelho de [`ies processo concluir`](processo-concluir.md):
mesma abertura, mesma verificação por `aberto_em`, mesmas invariantes de
escrita do [contrato da CLI](../cli.md) (prévia, confirmação e
verificação; escrita não confirmada nunca se repete; saídas 0 sucesso, 1 falha, 2
uso inválido, 3 escrita não confirmada). Só a Web, a mesma sessão; nenhuma URL
montada (só as publicadas pela barra e pela tela do nó). Estrutura
observada em [reabertura do processo](../../docs/como-o-sei-funciona/reabertura-do-processo.md).

## Entrada

- `NUP`: o mesmo de `ies processo NUP`; inválido sai com 2
  (erro do clap) antes de qualquer arquivo ou requisição.
- `--unidade SIGLA` (global): conferida na tela do processo, como na leitura.
- `--confirmar`: reabre. Sem ele, só prévia.

## Comportamento

1. **Abre o processo** como `processo concluir`: o caminho de
   [`ies processo NUP`](processo-obter-arvore.md) sem andamentos,
   propriedades nem marcadores, **pulando o link guardado da árvore**
   (ele não publica a unidade; a reabertura vale para a unidade de
   `lnkInfraUnidade` da tela do processo).
2. **Lê a situação** (`aberto_em`) e a **ação da barra**:
   `<a href="#" onclick="reabrirProcesso();">` (a mesma nas sete
   instalações), com o `href` assinado do nó raiz (`arvore_visualizar`)
   para achar a URL.
3. **Decide antes de escrever**, nesta ordem:

| Situação lida | Barra | Resultado |
| --- | --- | --- |
| não lida (sobrestado, anexado, fora da forma) | qualquer | saída 1, `SituacaoDoProcessoNaoLidaAntesDeReabrir` (`motivo`) |
| qualquer | fora da forma, repetida, raiz sem `href` | saída 1, subcódigo da barra |
| com a unidade da sessão | sem reabrir | saída 0, `alterado: false`, `reabertura_oferecida: false` (já aberto nela), sem escrita |
| com a unidade da sessão | com reabrir | saída 1, `UnidadeDaSessaoJaNaSituacao` |
| sem a unidade da sessão | sem reabrir | saída 1, `ReaberturaNaoPublicadaNaBarra` |
| sem a unidade da sessão | com reabrir | prévia (saída 0) ou escrita |

4. **Sem `--confirmar`**: publica `unidade`, `aberto_em` (siglas) e o plano
   de requisições. **Nenhuma** requisição além da abertura: a tela do nó
   não é pedida (a URL de lá já é a escrita).
5. **Com `--confirmar`**, uma escrita e nenhum retry: GET do
   `arvore_visualizar` da raiz; a resposta precisa ser HTML 200 com a URL
   `procedimento_reabrir` assinada do mesmo processo em **uma** de duas
   formas, senão saída 1 sem escrita:
   - `funcao` (SEI 4.0): o corpo de `function reabrirProcesso(){` na forma
     de três linhas de `concluirProcesso()` (função, `confirm` comentado,
     `location.href = '…';`);
   - `variavel` (SEI 4.1 e 5.0): a linha `var linkReabrirProcesso = '…';`.

   **Um GET** dessa URL: esse GET é a escrita.
6. **Verifica** por uma leitura nova, a de `ies processo NUP` (o passo mais
   barato que o índice permitir, em geral o link da árvore: 1 GET):
   sucesso só se a situação lida lista a unidade da sessão.

| Fato na tela do nó | Erro (stderr) |
| --- | --- |
| Nenhuma das duas formas | `ErroEnderecoDeReabrir::EnderecoDeReabrirAusenteNaTelaDoNo` |
| Função fora da forma exata | `ErroEnderecoDeReabrir::FuncaoDeReabrirForaDaFormaObservada(FuncaoForaDaForma)` |
| Variável fora da forma exata | `ErroEnderecoDeReabrir::EnderecoDeReabrirForaDaFormaObservada` |
| Duas funções, duas variáveis, ou as duas formas | `ErroEnderecoDeReabrir::EnderecosDeReabrirRepetidosNaTelaDoNo` |
| URL sem `procedimento_reabrir` assinado | `ErroEnderecoDeReabrir::EnderecoDaReaberturaNaoReconhecido` |
| URL de outro `id_procedimento` | `EnderecoDaReaberturaDeOutroProcesso` |

## Resposta e escrita não confirmada

**Esperada**: HTML 200 sem tela de login nem validação da Infra (nas
gravações, uma página no molde de `arvore_visualizar`). Qualquer outra
(rede, tempo, status, 302 — `RedirecionamentoNaReabertura` —, login,
validação) é `falha_do_envio` e **não** repete a escrita: a leitura de
verificação reconcilia, uma vez.

| Resposta | Situação lida depois | Saída | Resultado |
| --- | --- | --- | --- |
| Esperada | com a unidade da sessão | 0 | `alterado: true` |
| Não esperada | com a unidade da sessão | 0 | `alterado: true`, `reconciliado_por_leitura: true`, `falha_do_envio` |
| Qualquer | sem a unidade da sessão | 1 | `ReaberturaNaoAplicada` (com `falha_do_envio`, se houver) |
| Qualquer | leitura falhou ou situação não lida | 1 | `NaoConfirmadoNaReabertura` (campo `verificacao`: `VerificacaoPorLeitura`) |

Um SEI que processe a escrita depois da leitura não é detectável: antes de
repetir à mão, conferir com `ies processo NUP`. Falha **antes** da escrita
(abertura, barra, tela do nó, cookie) não é escrita: saída 1.

## Efeito

Abre o andamento do processo na unidade da sessão, **atribuído** ao
usuário (observado nas 11 reaberturas gravadas); as outras unidades
continuam como estavam. Desfaz-se com `ies processo concluir`.

## Saída

```json
{"nup": "00001.000001/2026-01", "id_procedimento": "10", "unidade": "TESTE",
 "origem": "web", "confirmado": true, "alterado": true,
 "reabertura_oferecida": true, "forma": "variavel", "aberto_em": [],
 "depois": ["TESTE"], "reconciliado_por_leitura": false, "requisicoes": 6}
```

`forma` é `funcao` ou `variavel`, omitida antes de pedir a tela do nó (na
prévia e quando não há o que fazer). `depois` são as siglas lidas na
verificação. Nunca URL, `infra_hash` ou cookie. Falhas: saída 1, stdout
vazio e `Error: <ErroReabrir::…>` em stderr; `aberto_em`,
`depois`, `falha_do_envio` e `verificacao` são campos dos erros que
os observaram, e o status HTTP vem na causa `Transporte(Status(..))`. A
saída humana termina com "Para reabrir, repita com --confirmar" na prévia.

## Sequência e orçamento

Abertura = a de `processo concluir`: 3 GET a frio pela linha do Controle
(Controle, tela do processo, árvore), 2 GET pelo link de trabalho
guardado; uma pasta fechada soma 1. Verificação = a leitura mais barata
(1 GET pelo link da árvore que a abertura guardou).

| Caso | Requisições |
| --- | --- |
| Uso inválido | 0 |
| Prévia, nada a fazer ou recusa antes da escrita | abertura |
| Tela do nó recusada | abertura + 1 GET |
| `--confirmar` (qualquer resultado da verificação) | abertura + 2 GET + verificação |

A frio, com a árvore guardada pela abertura: 6 GET; pelo link de trabalho
guardado: 5 GET. Os testes conferem a sequência exata.

## Fora do escopo

Reabrir noutra unidade sem trocar a sessão, reabertura programada
("Concluir e agendar reabertura" é da tela de conclusão), reabrir vários
processos, escolher a quem fica atribuído.

## Evidência

[Evidência da operação](../evidencias/processo-reabrir.md) (Todos, 11
gravações nas sete instalações), [observação estrutural](../../docs/como-o-sei-funciona/reabertura-do-processo.md),
[decisão](../../docs/decisoes/processo-reabrir.md),
[replay do acervo](../../docs/validacao-replay/2026-11-04-processo-reabrir-acervo.md)
e [validação live](../../docs/validacao-live/2026-11-04-processo-concluir.md)
(junto com a de concluir).
