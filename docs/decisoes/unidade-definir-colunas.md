---
type: ImplementationDecision
title: Escrita explícita, única e verificada das colunas do Nível de detalhe
operation: unidade.definir_colunas
implementation: partial
status: adopted-for-scope
scope: Nível de detalhe do Controle de Processos Web com sessão existente; ies unidade colunas --definir todas|LISTA com prévia, um POST de salvar com --confirmar, verificação por um GET; vinda da R, a sessão fica na D
selected_strategy: web-escrita-explicita-unica
alternatives: web-escrita-explicita-unica,web-salvar-e-restaurar-por-listagem,rest-wssei,soap-official
evidence_level: ensaio-live-do-proprietario-e-testes-sinteticos
live_validation: nao-realizada-neste-recorte
---

# Colunas do Nível de detalhe por escrita explícita

## Alternativas consideradas

- **web-salvar-e-restaurar-por-listagem (opção A):** a cada
  `listar-processos`, marcar todas as opções, ler a D e restaurar a
  configuração encontrada. Leitura sempre com o máximo de colunas, sem
  mudar a preferência do usuário ao fim.
- **web-escrita-explicita-unica (opção B, escolhida):** o máximo de colunas
  é aplicado **uma vez**, por comando explícito,
  `ies unidade colunas --definir todas --confirmar`, com prévia, um POST de
  salvar e verificação por leitura. As leituras não escrevem.
- **rest-wssei / soap-official:** não há evidência de serviço que leia ou
  grave a preferência de colunas do Controle; não avaliados além do registro.

## Comparação

A opção A transforma toda leitura em duas escritas de preferência (salvar e
restaurar), com dois pontos de efeito incerto por listagem: um timeout na
restauração deixaria a preferência do usuário alterada sem pedido, e a
leitura, que hoje não escreve, passaria a exigir confirmação. Custa 2 POSTs e
2 GETs a mais por `listar-processos`. A opção B concentra o risco num comando
que o usuário pede, com prévia e confirmação, e mantém as leituras sem
escrita (`colunas_ausentes` aponta o comando). Em troca, a preferência fica
alterada **de propósito** e vale também no navegador do usuário; a volta é
outro `--definir` com a lista de antes, que a saída publica.

## Decisão

Decisão do proprietário "B" (02/11/2026). Implementado só na Web, na mesma
sessão, sem troca de backend:

- Erro de uso (lista vazia, item vazio, chave desconhecida ou repetida) sai
  com 2 antes de ler a configuração local, sem requisição.
- Leitura pelo mesmo fluxo de `ies unidade colunas` (link da D; vinda da R,
  troca para D e fica nela) e conferência do formulário **também na prévia**:
  controle enviável fora de `hdnInfraTipoPagina`, `sbmSalvar` e das caixas
  do catálogo é recusa; a derivação do primitivo exige `sbmSalvar` único.
- `todas` é o que a instalação publica (UnB 4.0.9: 11); coluna pedida e não
  publicada é recusa antes do POST; alvo igual ao atual não envia nada.
- A `const` `SALVAR_NIVEL_DE_DETALHE` fica no código da operação
  (`src/nivel_de_detalhe/salvar.rs`), sem despachante. Duas declarações
  novas do primitivo, por operação: `ConferenciaDaUnidade::NaTelaDeOrigemDoLink`
  (a janela não publica `lnkInfraUnidade`; a unidade é a da D conferida na
  mesma sessão, e se a janela publicar o rótulo ele também é conferido) e
  `ResultadoDoEnvio::ConferidoPorLeituraPosterior` (não há marcador de
  sucesso observado; HTML 200 sem login é a resposta esperada, e o efeito
  só é afirmado pela leitura).
- Efeito novo `EfeitoAvaliado::AlteraPreferenciaDoUsuario`
  (`ColunasDaVisaoDetalhada`): preferência persistente, distinta do estado
  de sessão.
- Um POST, sem retry; **um** GET da mesma URL para verificar. Resposta não
  esperada reconcilia pela mesma leitura (tabela no
  [contrato](../../contracts/cli/unidade-definir-colunas.md)): alvo →
  sucesso com `reconciliado_por_leitura`; antes → não aplicada (1); outro,
  ou leitura impossível → efeito incerto (3).
- Sem volta à R (decisão do proprietário, 02/11/2026, registrada na
  [decisão da leitura](unidade-listar-processos.md)): restaurar a visão
  gastava um POST por execução e criava um terceiro desfecho (escrita feita,
  visão não restaurada) que a saída precisava publicar ao lado da escrita.
  Vinda da R: 2 GET + 1 POST na prévia, 3 GET + 2 POST ao salvar. A PEN1
  manteve a D na sessão e num login novo da mesma conta (efeito cosmético);
  a UnB 4.0.9 volta à R no GET seguinte. A semântica da escrita (prévia,
  `--confirmar`, verificação, efeito incerto) não muda.

Descartado: aceitar o formulário com campos desconhecidos (enviaria um POST
diferente do observado) e classificar a resposta por um marcador inventado
(o HTML exato não foi registrado). `ies setup` não aplica `--definir todas`
nesta PR; fica como integração futura possível.

## Evidências verificáveis

- [Contrato da operação](../../contracts/cli/unidade-definir-colunas.md) e
  [contrato da leitura](../../contracts/cli/unidade-colunas.md)
- [Evidência da operação](../../contracts/evidencias/unidade-definir_colunas.md)
- [Ensaio live do proprietário](../validacao-live/2026-11-02-leitura-controle.md):
  POST de salvar na PEN1 5.0.4 (só o oculto, o botão e as caixas `on`;
  200 sem redirect; efeito imediato na D e no GET da tela)
- [Testes da superfície da CLI](../../tests/cli_unidade_listas.rs)
- [Fluxo, sequência e reconciliação](../../src/visao_do_controle/tests_definir.rs),
  [alvo, plano e formulário](../../src/nivel_de_detalhe/tests_alvo.rs),
  [primitivo](../../src/formulario_web/tests_conferencia.rs) e
  [saída JSON](../../src/unidade_colunas_definir_cli/tests.rs)
- [Roteiro live](../validacao-live/roteiro-definir-colunas.md)

## Limitações e revisão

Testes só com telas sintéticas modeladas na estrutura observada (12 e 11
opções); live do comando **NOT_RUN**. A resposta de sucesso não tem marcador;
um servidor que processe o POST depois do GET de verificação não é
detectável. Revisar se o ensaio live mostrar: `lnkInfraUnidade` na janela,
mais de um `sbmSalvar`, controle além dos observados, resposta diferente de
200, mudança que só aparece depois de outra navegação, limite de opções
marcadas (`validarNumeroCriterios` no `onchange` das caixas não foi
capturado), ou outro catálogo de opções. Capturar a estrutura da resposta
do salvar para, se estável, trocar a leitura obrigatória por um marcador
mais a leitura.
