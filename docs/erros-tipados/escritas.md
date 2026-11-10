---
type: Guide
title: Erros das escritas
description: Concluir, reabrir, assinar e enviar o processo
---

# Erros das escritas

Parte do [guia de erros tipados](../erros-tipados.md), separada por responsabilidade.

## Concluir o processo

`ErroConcluir` (`src/processo_concluir/erro.rs`) separa o que impede a
escrita (abertura, situação não lida, barra, tela, função, derivação:
saída 1, nada enviado) do resultado depois dela: `ConclusaoNaoAplicada`
(a leitura ainda lista a unidade) e `EfeitoIncertoNaConclusao` (saída 1,
com `VerificacaoPorLeitura`). `FalhaDoEnvio` classifica a resposta não
esperada da escrita sem decidir o efeito: quem decide é a leitura, uma vez,
sem retry ([contrato](../../contracts/cli/processo-concluir.md)).

## Reabrir o processo

`ErroReabrir` (`src/processo_reabrir/erro.rs`) segue a mesma divisão: o
que impede a escrita (abertura, situação não lida, barra, tela do nó,
cookie: saída 1, nada enviado) e o resultado depois dela,
`ReaberturaNaoAplicada` (a leitura não lista a unidade) e
`EfeitoIncertoNaReabertura` (saída 1). A resposta não esperada é a mesma
`FalhaDoEnvio` de concluir, com a variante `RedirecionamentoNaReabertura`;
a verificação é a mesma `VerificacaoPorLeitura`
([contrato](../../contracts/cli/processo-reabrir.md)).

## Assinar o documento (Refs #174)

`ErroAssinar` (`src/documento_assinar/erro.rs`) separa o que impede a
escrita (abertura, credenciais, `ErroAssinaturaNaArvore` da ação e da
caneta, tela do documento, `ErroLinkDeAssinar`, `ErroTelaDeAssinatura`,
`cargo_nao_oferecido_pela_tela`, `documento_assinado_por_outro_usuario`,
derivação: saída 1, nada enviado) do resultado depois dela:
`AssinaturaNaoAplicada` (a caneta relida é a de antes) e
`EfeitoIncertoNaAssinatura` (saída 1, com `VerificacaoDaAssinatura`).
`FalhaDoEnvio` da assinatura (`redirecionamento_apos_assinar`,
`resposta_da_assinatura_sem_finalizacao` ou o erro do primitivo) não
decide o efeito: a leitura decide, uma vez, sem retry
([contrato](../../contracts/cli/documento-assinar.md)).

## Marcar processo (Refs #255)

`ErroMarcar` (`src/processo_marcar.rs`) separa erro de uso (`id#nome`
inválido), leitura insuficiente (lista parcial, unidade ausente, tela não
publicada), identidade divergente (`id` visto com outro nome) e
capacidade ainda não observada (catálogo da unidade ou formulário de
remoção). Nenhum desses estados autoriza POST. O leitor Web preserva
status e mensagem saneada de recusas HTTP. Ver
[contrato](../../contracts/cli/processo-marcar.md).

## Enviar o processo

`ErroEnviar` (`src/processo_enviar/erro.rs`) segue a divisão de concluir:
antes do POST (abertura, situação, barra `ErroEnviarNaBarra`, tela
`ErroTelaDeEnvio`, consulta `FalhaDaConsulta`, destino não oferecido ou
repetido, já aberto no destino, derivação: saída 1, nada enviado) e depois
dele: `EnvioNaoAplicado` (sem o destino), `SituacaoDaOrigemDiferenteDaPedidaAposEnvio`
(o destino abriu, mas a unidade da sessão não ficou como `--manter-aberto`
pediu) e `EfeitoIncertoNoEnvio` (saída 1). A resposta não esperada é a
`FalhaDoEnvio` de concluir, com `RedirecionamentoNaoReconhecidoAposEnviar`
e `RespostaDoEnvioSemRedirecionamento`
([contrato](../../contracts/cli/processo-enviar.md)).
