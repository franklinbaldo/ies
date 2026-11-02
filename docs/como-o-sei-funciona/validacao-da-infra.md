---
type: SEIObservation
title: Mensagem de validação da Infra
description: O textarea oculto txaInfraValidacao com que o SEI recusa um POST respondendo HTTP 200
---

# Mensagem de validação da Infra (`txaInfraValidacao`)

## Observação

UnB 4.0.9, treinamento, 02/11/2026, uma tentativa de `ies login` com
senha errada: o POST do `frmLogin` respondeu **HTTP 200** com a tela de
login de novo (`txtUsuario` já preenchido) e, fora do formulário:

```html
<textarea id="txaInfraValidacao" name="txaInfraValidacao" style="display:none">Usuário ou Senha Inválida.</textarea>
<script type="text/javascript"><!--//--><![CDATA[//><!--
self.setTimeout('alert(document.getElementById(\'txaInfraValidacao\').value)',300);
//--><!]]></script>
```

O navegador mostra o texto num `alert` 300 ms depois de carregar. Não
há `alert-danger`, `divInfraExcecao` nem `infraMensagemErro` com o
texto: quem lê só esses contêineres não vê a recusa.

## Inferência (não verificada)

O prefixo `txaInfra` e o `alert` genérico sugerem o mecanismo de
mensagem de validação do framework Infra do SEI, usado em qualquer POST
recusado por regra de negócio, não só no login do SIP. Só o login da
UnB foi observado. Não se sabe se mensagens de **sucesso** usam o mesmo
textarea; nenhuma cassette do acervo traz `txaInfraValidacao`.

## Como o IES lê

- Por `id`, com `tl` e `html_atributos`, sem regex; texto do primeiro
  `#txaInfraValidacao` não vazio, entidades decodificadas (inclusive as
  letras acentuadas do português por nome), saneado e cortado em 500
  caracteres como as demais mensagens do servidor.
- `ies login`: `credencial_recusada` traz esse texto em
  `mensagem_do_servidor`; sem ele, valem os contêineres anteriores.
- Texto das recusas HTTP: `txaInfraValidacao` antes de `divInfraExcecao`.
- Primitivo de formulário (troca de visão, `--pagina N`, `colunas
  --definir`, pesquisa): resposta 200 com o textarea não vazio é
  `validacao_recusada_pelo_sei`, com ou sem o marcador de resultado da
  operação. Textarea vazio não muda nada.

## Limites

Só a estrutura acima e o texto genérico da recusa de credencial foram
observados. Se uma tela de sucesso trouxer o textarea preenchido, o IES
a recusará: o efeito de uma escrita fica incerto e vai para a leitura de
verificação, nunca para um novo envio. Revalidar nas outras instalações.
